use std::collections::{BTreeMap, HashMap};

use crate::encoding::TextEncoding;
use crate::hash::{hex, md5};
use crate::io_util::{checked_add, checked_mul, put_u16, put_u32, read_u16, read_u32};
use crate::json_model::{LinkMeta, TranslationEntry, TranslationFile, FORMAT_ID};
use crate::markup::{absolute_to_line_col, insert_link_markup, parse_link_markup, ParsedMarkup};
use crate::ToolResult;

const HEADER_SIZE: usize = 304;
const INSTRUCTION_SIZE: usize = 12;
const TRAILING_MD5_SIZE: usize = 16;
const MAX_TEXT_PAYLOAD: usize = 2044;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptEntry {
    pub scenario_id: u32,
    pub entry_index: u32,
    pub code_offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction {
    pub absolute_offset: usize,
    pub code_offset: usize,
    pub opcode: u16,
    pub flags: u16,
    pub operand: u32,
}

#[derive(Debug, Clone)]
pub struct LinkControl {
    pub instruction: Instruction,
    pub line: u16,
    pub first: u16,
    pub len: u16,
    pub variable: u32,
    pub target_entry: u32,
    pub trailing: u32,
    pub target: ScriptEntry,
}

#[derive(Debug, Clone)]
pub struct RubyControl {
    pub instruction: Instruction,
}

#[derive(Debug, Clone)]
pub struct FontControl {
    pub instruction: Instruction,
    pub line: u16,
}

#[derive(Debug, Clone)]
pub struct TextRecord {
    pub instruction: Instruction,
    pub record_offset: usize,
    pub record_size: usize,
    pub ident: u32,
    pub meta: u32,
    pub parts: Vec<String>,
    pub ruby_tail: Vec<u8>,
    pub ruby_count: u32,
    pub links: Vec<LinkControl>,
    pub rubies: Vec<RubyControl>,
    pub fonts: Vec<FontControl>,
}

#[derive(Debug, Clone)]
pub struct ChoiceOption {
    pub absolute_offset: usize,
    pub byte_size: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ChoiceGroup {
    pub instruction: Instruction,
    pub prefix: Vec<u8>,
    pub options: Vec<ChoiceOption>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryRef {
    Text(usize),
    Choice { group: usize, option: usize },
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub bytes: Vec<u8>,
    pub scenario_id: u32,
    pub title: String,
    pub entries: Vec<ScriptEntry>,
    pub instructions: Vec<Instruction>,
    pub text_records: Vec<TextRecord>,
    pub choice_groups: Vec<ChoiceGroup>,
    pub code_start: usize,
    pub code_size: usize,
    pub data_start: usize,
    pub data_size: usize,
    pub body_end: usize,
}

#[derive(Debug, Default, Clone)]
pub struct ApplyStats {
    pub patched: u64,
    pub unchanged: u64,
    pub removed_join_spaces: u64,
    pub ambiguous_join_spaces: u64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct ResolvedLink {
    id: u32,
    anchor_record: usize,
    link_index: usize,
    target_record: usize,
    source_start: usize,
    source_len: usize,
}

#[derive(Debug)]
struct PreparedTextRow {
    json_index: u32,
    markup: ParsedMarkup,
    entry_changed: bool,
    text_changed: bool,
    rebuilt: bool,
    removed_join_spaces: u64,
    ambiguous_join_spaces: u64,
}

pub fn identity_map(max_id: u32) -> HashMap<[u8; 16], u32> {
    let mut map = HashMap::with_capacity((max_id as usize).saturating_add(1));
    for id in 0..=max_id {
        let mut input = b"Farthest_vo_dl".to_vec();
        input.extend_from_slice(&(id as i32).to_le_bytes());
        map.insert(md5(&input), id);
    }
    map
}

impl Scenario {
    pub fn parse(
        bytes: Vec<u8>,
        text_encoding: TextEncoding,
        identities: &HashMap<[u8; 16], u32>,
        context: &str,
    ) -> ToolResult<Self> {
        if bytes.len() < HEADER_SIZE + TRAILING_MD5_SIZE {
            return Err(format!("{context}: too small to be a numbered scenario"));
        }
        let identity: [u8; 16] = bytes[0..16].try_into().unwrap();
        let scenario_id = *identities.get(&identity).ok_or_else(|| {
            format!("{context}: first 16 bytes are not a recognized scenario identity MD5")
        })?;
        let expected_tail = md5(&bytes[..bytes.len() - TRAILING_MD5_SIZE]);
        if bytes[bytes.len() - TRAILING_MD5_SIZE..] != expected_tail {
            return Err(format!("{context}: trailing content MD5 mismatch"));
        }

        let mut title_bytes = bytes[32..292].to_vec();
        for byte in &mut title_bytes {
            *byte ^= 0x16;
        }
        let title_end = title_bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(title_bytes.len());
        let title =
            TextEncoding::Cp932.decode(&title_bytes[..title_end], &format!("{context}: title"))?;

        let entry_count = read_u32(&bytes, 292, context)? as usize;
        let code_size = read_u32(&bytes, 296, context)? as usize;
        let data_size = read_u32(&bytes, 300, context)? as usize;
        if !code_size.is_multiple_of(INSTRUCTION_SIZE) {
            return Err(format!(
                "{context}: code_size 0x{code_size:X} is not divisible by 12"
            ));
        }
        let table_size = checked_mul(entry_count, 12, context)?;
        let code_start = checked_add(HEADER_SIZE, table_size, context)?;
        let data_start = checked_add(code_start, code_size, context)?;
        let body_end = checked_add(data_start, data_size, context)?;
        let expected_size = checked_add(body_end, TRAILING_MD5_SIZE, context)?;
        if expected_size != bytes.len() {
            return Err(format!(
                "{context}: size mismatch; header describes {expected_size} bytes, file has {}",
                bytes.len()
            ));
        }

        let mut entries = Vec::with_capacity(entry_count);
        for index in 0..entry_count {
            let offset = HEADER_SIZE + index * 12;
            entries.push(ScriptEntry {
                scenario_id: read_u32(&bytes, offset, context)?,
                entry_index: read_u32(&bytes, offset + 4, context)?,
                code_offset: read_u32(&bytes, offset + 8, context)?,
            });
        }

        let mut instructions = Vec::with_capacity(code_size / INSTRUCTION_SIZE);
        for code_offset in (0..code_size).step_by(INSTRUCTION_SIZE) {
            let absolute_offset = code_start + code_offset;
            instructions.push(Instruction {
                absolute_offset,
                code_offset,
                opcode: read_u16(&bytes, absolute_offset, context)?,
                flags: read_u16(&bytes, absolute_offset + 2, context)?,
                operand: read_u32(&bytes, absolute_offset + 4, context)?,
            });
        }

        let mut text_records = Vec::new();
        let mut choice_groups = Vec::new();
        let mut pending_links = Vec::new();
        let mut pending_rubies = Vec::new();
        let mut pending_fonts = Vec::new();

        for instruction in &instructions {
            match instruction.opcode {
                0x44 => pending_rubies.push(parse_ruby_control(
                    &bytes,
                    data_start,
                    data_size,
                    *instruction,
                    context,
                )?),
                0x46 => pending_links.push(parse_link_control(
                    &bytes,
                    data_start,
                    data_size,
                    *instruction,
                    &entries,
                    context,
                )?),
                0x49 => pending_fonts.push(parse_font_control(
                    &bytes,
                    data_start,
                    data_size,
                    *instruction,
                    context,
                )?),
                0x40 | 0x41 => {
                    let record = parse_text_record(
                        &bytes,
                        data_start,
                        data_size,
                        *instruction,
                        text_encoding,
                        context,
                    )?;
                    text_records.push(TextRecord {
                        links: std::mem::take(&mut pending_links),
                        rubies: std::mem::take(&mut pending_rubies),
                        fonts: std::mem::take(&mut pending_fonts),
                        ..record
                    });
                }
                0x61..=0x65 => choice_groups.push(parse_choice_group(
                    &bytes,
                    data_start,
                    data_size,
                    *instruction,
                    text_encoding,
                    context,
                )?),
                _ => {}
            }
        }
        if !pending_links.is_empty() || !pending_rubies.is_empty() || !pending_fonts.is_empty() {
            return Err(format!(
                "{context}: dangling text controls at end of instruction stream (links {}, ruby {}, fonts {})",
                pending_links.len(),
                pending_rubies.len(),
                pending_fonts.len()
            ));
        }

        Ok(Self {
            bytes,
            scenario_id,
            title,
            entries,
            instructions,
            text_records,
            choice_groups,
            code_start,
            code_size,
            data_start,
            data_size,
            body_end,
        })
    }

    pub fn export(
        &self,
        relative_file: &str,
        source_encoding: TextEncoding,
    ) -> ToolResult<(TranslationFile, Vec<EntryRef>)> {
        let mut rows: Vec<(usize, Vec<(TranslationEntry, EntryRef)>)> = Vec::new();
        let resolved_links = self.resolve_links(relative_file)?;

        for (record_index, record) in self.text_records.iter().enumerate() {
            let (name, body_parts) = split_name_body(record, relative_file)?;
            let plain = body_parts.concat();
            let mut spans = Vec::new();
            let mut link_meta = Vec::new();
            for resolved in &resolved_links[record_index] {
                let link = &self.text_records[resolved.anchor_record].links[resolved.link_index];
                spans.push((resolved.id, resolved.source_start, resolved.source_len));
                link_meta.push(LinkMeta {
                    id: resolved.id,
                    inst_offset: link.instruction.code_offset as u64,
                    target_scenario: link.target.scenario_id,
                    target_entry: link.target.entry_index,
                    target_code_offset: link.target.code_offset,
                    source_line: link.line,
                    source_first: link.first,
                    source_len: link.len,
                    inline: true,
                });
            }
            let marked = insert_link_markup(
                &plain,
                &spans,
                &format!(
                    "{relative_file}: instruction 0x{:X}",
                    record.instruction.code_offset
                ),
            )?;
            let row = TranslationEntry {
                file: relative_file.to_string(),
                index: 0,
                offset: record.record_offset as u64,
                inst_offset: record.instruction.code_offset as u64,
                size: u32::try_from(record.record_size)
                    .map_err(|_| format!("{relative_file}: text record size exceeds u32"))?,
                entry_type: if name.is_some() {
                    "dialogue".to_string()
                } else {
                    "monologue".to_string()
                },
                opcode: format!("0x{:02X}", record.instruction.opcode),
                encoding: source_encoding.label().to_string(),
                policy: "append_relocate".to_string(),
                flags: record.instruction.flags,
                choice_index: None,
                scr_name: name.clone(),
                name,
                scr_msg: marked.clone(),
                message: marked,
                links: link_meta,
            };
            rows.push((
                record.instruction.code_offset,
                vec![(row, EntryRef::Text(record_index))],
            ));
        }

        for (group_index, group) in self.choice_groups.iter().enumerate() {
            let mut group_rows = Vec::new();
            for (option_index, option) in group.options.iter().enumerate() {
                let row = TranslationEntry {
                    file: relative_file.to_string(),
                    index: 0,
                    offset: option.absolute_offset as u64,
                    inst_offset: group.instruction.code_offset as u64,
                    size: u32::try_from(option.byte_size)
                        .map_err(|_| format!("{relative_file}: choice size exceeds u32"))?,
                    entry_type: "choice".to_string(),
                    opcode: format!("0x{:02X}", group.instruction.opcode),
                    encoding: source_encoding.label().to_string(),
                    policy: "append_relocate".to_string(),
                    flags: group.instruction.flags,
                    choice_index: Some(option_index as u16),
                    scr_name: None,
                    name: None,
                    scr_msg: option.text.clone(),
                    message: option.text.clone(),
                    links: Vec::new(),
                };
                group_rows.push((
                    row,
                    EntryRef::Choice {
                        group: group_index,
                        option: option_index,
                    },
                ));
            }
            rows.push((group.instruction.code_offset, group_rows));
        }

        rows.sort_by_key(|row| row.0);
        let mut json_entries = Vec::new();
        let mut refs = Vec::new();
        for (_, group) in rows {
            for (mut row, entry_ref) in group {
                row.index = json_entries.len() as u32;
                json_entries.push(row);
                refs.push(entry_ref);
            }
        }
        let file = TranslationFile {
            format: FORMAT_ID.to_string(),
            file: relative_file.to_string(),
            scenario_id: self.scenario_id,
            title: self.title.clone(),
            source_md5: hex(&md5(&self.bytes)),
            entries: json_entries,
        };
        Ok((file, refs))
    }

    fn resolve_links(&self, context: &str) -> ToolResult<Vec<Vec<ResolvedLink>>> {
        let mut by_target = vec![Vec::new(); self.text_records.len()];

        for (anchor_record, record) in self.text_records.iter().enumerate() {
            for (link_index, link) in record.links.iter().enumerate() {
                if link.line == 0 || link.first == 0 || link.len == 0 {
                    return Err(format!(
                        "{context}: hyperlink at instruction 0x{:X} uses a zero coordinate",
                        link.instruction.code_offset
                    ));
                }

                let mut remaining_line = link.line as usize;
                let mut previous_text_offset = record.instruction.code_offset;
                let mut resolved = None;

                for target_record in anchor_record..self.text_records.len() {
                    let target = &self.text_records[target_record];
                    if target_record > anchor_record {
                        let crosses_group_end = self.instructions.iter().any(|instruction| {
                            instruction.code_offset > previous_text_offset
                                && instruction.code_offset < target.instruction.code_offset
                                && instruction.opcode == 0x53
                        });
                        if crosses_group_end {
                            break;
                        }
                    }

                    let (_, body_parts) = split_name_body(target, context)?;
                    if remaining_line <= body_parts.len() {
                        let local_line = remaining_line - 1;
                        let line = &body_parts[local_line];
                        let first = link.first as usize - 1;
                        let len = link.len as usize;
                        let end = first.checked_add(len).ok_or_else(|| {
                            format!(
                                "{context}: hyperlink at instruction 0x{:X} range overflow",
                                link.instruction.code_offset
                            )
                        })?;
                        let line_len = line.chars().count();
                        if end > line_len {
                            return Err(format!(
                                "{context}: hyperlink at instruction 0x{:X} range {}+{} exceeds group line {} length {}",
                                link.instruction.code_offset,
                                link.first,
                                link.len,
                                link.line,
                                line_len
                            ));
                        }
                        let prior = body_parts[..local_line]
                            .iter()
                            .map(|value| value.chars().count())
                            .sum::<usize>();
                        resolved = Some((target_record, prior + first, len));
                        break;
                    }

                    remaining_line -= body_parts.len();
                    previous_text_offset = target.instruction.code_offset;
                }

                let (target_record, source_start, source_len) = resolved.ok_or_else(|| {
                    format!(
                        "{context}: hyperlink at instruction 0x{:X} line {} does not resolve before the display-group terminator",
                        link.instruction.code_offset, link.line
                    )
                })?;
                let id = u32::try_from(by_target[target_record].len())
                    .map_err(|_| format!("{context}: too many hyperlinks in one text entry"))?;
                by_target[target_record].push(ResolvedLink {
                    id,
                    anchor_record,
                    link_index,
                    target_record,
                    source_start,
                    source_len,
                });
            }
        }

        Ok(by_target)
    }
}

fn split_name_body<'a>(
    record: &'a TextRecord,
    context: &str,
) -> ToolResult<(Option<String>, &'a [String])> {
    match record.instruction.flags {
        0x12 => Ok((None, &record.parts)),
        0x13 => {
            let (name, body) = record
                .parts
                .split_first()
                .ok_or_else(|| format!("{context}: flag 0x13 text has no name slot"))?;
            Ok((Some(name.clone()), body))
        }
        flag => Err(format!(
            "{context}: text instruction 0x{:X} has unsupported flag 0x{flag:X}",
            record.instruction.code_offset
        )),
    }
}

fn data_absolute(
    data_start: usize,
    data_size: usize,
    relative: u32,
    minimum: usize,
    context: &str,
) -> ToolResult<usize> {
    let relative = relative as usize;
    let end = checked_add(relative, minimum, context)?;
    if end > data_size {
        return Err(format!(
            "{context}: data reference 0x{relative:X} + 0x{minimum:X} exceeds data_size 0x{data_size:X}"
        ));
    }
    checked_add(data_start, relative, context)
}

fn parse_text_record(
    bytes: &[u8],
    data_start: usize,
    data_size: usize,
    instruction: Instruction,
    encoding: TextEncoding,
    context: &str,
) -> ToolResult<TextRecord> {
    let start = data_absolute(data_start, data_size, instruction.operand, 16, context)?;
    let ident = read_u32(bytes, start, context)?;
    let text_size = read_u32(bytes, start + 4, context)? as usize;
    if text_size < 8 {
        return Err(format!("{context}: text section is smaller than 8 bytes"));
    }
    let text_end = checked_add(start + 8, text_size, context)?;
    let data_end = data_start + data_size;
    if text_end + 4 > data_end {
        return Err(format!("{context}: text section exceeds data region"));
    }
    let meta = read_u32(bytes, start + 8, context)?;
    let count = read_u32(bytes, start + 12, context)? as usize;
    let mut cursor = start + 16;
    let mut parts = Vec::with_capacity(count);
    for part_index in 0..count {
        let len = read_u32(bytes, cursor, context)? as usize;
        cursor = checked_add(cursor, 4, context)?;
        let end = checked_add(cursor, len, context)?;
        if end > text_end {
            return Err(format!(
                "{context}: text part {part_index} exceeds declared text section"
            ));
        }
        let mut decoded = bytes[cursor..end].to_vec();
        for byte in &mut decoded {
            *byte ^= 0x53;
        }
        parts.push(encoding.decode(
            &decoded,
            &format!(
                "{context}: text at instruction 0x{:X}, part {part_index}",
                instruction.code_offset
            ),
        )?);
        cursor = end;
    }
    if cursor != text_end {
        return Err(format!(
            "{context}: text section has {} unparsed bytes",
            text_end - cursor
        ));
    }
    let ruby_size = read_u32(bytes, text_end, context)? as usize;
    if ruby_size < 4 {
        return Err(format!("{context}: ruby section is smaller than 4 bytes"));
    }
    let record_end = checked_add(text_end + 4, ruby_size, context)?;
    if record_end > data_end {
        return Err(format!("{context}: ruby section exceeds data region"));
    }
    let ruby_count = read_u32(bytes, text_end + 4, context)?;
    Ok(TextRecord {
        instruction,
        record_offset: start,
        record_size: record_end - start,
        ident,
        meta,
        parts,
        ruby_tail: bytes[text_end..record_end].to_vec(),
        ruby_count,
        links: Vec::new(),
        rubies: Vec::new(),
        fonts: Vec::new(),
    })
}

fn parse_link_control(
    bytes: &[u8],
    data_start: usize,
    data_size: usize,
    instruction: Instruction,
    entries: &[ScriptEntry],
    context: &str,
) -> ToolResult<LinkControl> {
    let start = data_absolute(data_start, data_size, instruction.operand, 18, context)?;
    let line = read_u16(bytes, start, context)?;
    let first = read_u16(bytes, start + 2, context)?;
    let len = read_u16(bytes, start + 4, context)?;
    let variable = read_u32(bytes, start + 6, context)?;
    let target_entry = read_u32(bytes, start + 10, context)?;
    let trailing = read_u32(bytes, start + 14, context)?;
    let target = *entries.get(target_entry as usize).ok_or_else(|| {
        format!("{context}: hyperlink target entry 0x{target_entry:X} is outside entry table")
    })?;
    Ok(LinkControl {
        instruction,
        line,
        first,
        len,
        variable,
        target_entry,
        trailing,
        target,
    })
}

fn parse_ruby_control(
    bytes: &[u8],
    data_start: usize,
    data_size: usize,
    instruction: Instruction,
    context: &str,
) -> ToolResult<RubyControl> {
    let start = data_absolute(data_start, data_size, instruction.operand, 16, context)?;
    let reading_len = read_u32(bytes, start + 12, context)? as usize;
    let required = checked_add(16, reading_len, context)?;
    let _ = data_absolute(
        data_start,
        data_size,
        instruction.operand,
        required,
        context,
    )?;
    Ok(RubyControl { instruction })
}

fn parse_font_control(
    bytes: &[u8],
    data_start: usize,
    data_size: usize,
    instruction: Instruction,
    context: &str,
) -> ToolResult<FontControl> {
    let start = data_absolute(data_start, data_size, instruction.operand, 16, context)?;
    Ok(FontControl {
        instruction,
        line: read_u16(bytes, start, context)?,
    })
}

fn parse_choice_group(
    bytes: &[u8],
    data_start: usize,
    data_size: usize,
    instruction: Instruction,
    encoding: TextEncoding,
    context: &str,
) -> ToolResult<ChoiceGroup> {
    let count = instruction.flags as usize;
    let prefix_len = match instruction.opcode {
        0x61 | 0x62 => 0,
        0x63 => 4,
        0x64 => checked_mul(count, 4, context)?,
        0x65 => checked_add(4, checked_mul(count, 4, context)?, context)?,
        _ => return Err(format!("{context}: unsupported choice opcode")),
    };
    let start = data_absolute(
        data_start,
        data_size,
        instruction.operand,
        prefix_len,
        context,
    )?;
    let data_end = data_start + data_size;
    let mut cursor = start + prefix_len;
    let mut options = Vec::with_capacity(count);
    for option_index in 0..count {
        let len = read_u32(bytes, cursor, context)? as usize;
        cursor = checked_add(cursor, 4, context)?;
        let end = checked_add(cursor, len, context)?;
        if end > data_end {
            return Err(format!(
                "{context}: choice option {option_index} exceeds data region"
            ));
        }
        let mut decoded = bytes[cursor..end].to_vec();
        for byte in &mut decoded {
            *byte ^= 0x53;
        }
        options.push(ChoiceOption {
            absolute_offset: cursor,
            byte_size: len,
            text: encoding.decode(
                &decoded,
                &format!(
                    "{context}: choice at instruction 0x{:X}, option {option_index}",
                    instruction.code_offset
                ),
            )?,
        });
        cursor = end;
    }
    Ok(ChoiceGroup {
        instruction,
        prefix: bytes[start..start + prefix_len].to_vec(),
        options,
    })
}

impl Scenario {
    pub fn apply_translation(
        &self,
        relative_file: &str,
        source_encoding: TextEncoding,
        target_encoding: TextEncoding,
        translated: &TranslationFile,
    ) -> ToolResult<(Vec<u8>, ApplyStats)> {
        let (expected, refs) = self.export(relative_file, source_encoding)?;
        validate_translation_file(&expected, translated)?;

        let mut body = self.bytes[..self.body_end].to_vec();
        let mut appended = Vec::new();
        let mut stats = ApplyStats::default();
        let mut choice_rows: BTreeMap<usize, Vec<&TranslationEntry>> = BTreeMap::new();
        let mut text_json_indices = vec![None; self.text_records.len()];

        for (json_index, (row, entry_ref)) in translated.entries.iter().zip(refs.iter()).enumerate()
        {
            match *entry_ref {
                EntryRef::Text(record_index) => text_json_indices[record_index] = Some(json_index),
                EntryRef::Choice { group, .. } => {
                    choice_rows.entry(group).or_default().push(row);
                }
            }
        }

        let mut prepared_text = Vec::with_capacity(self.text_records.len());
        for (record_index, record) in self.text_records.iter().enumerate() {
            let json_index = text_json_indices[record_index].ok_or_else(|| {
                format!("{relative_file}: text record {record_index} has no JSON entry")
            })?;
            let expected_row = &expected.entries[json_index];
            let translated_row = &translated.entries[json_index];
            let prepared = prepare_text_row(record, expected_row, translated_row, relative_file)?;
            stats.removed_join_spaces += prepared.removed_join_spaces;
            stats.ambiguous_join_spaces += prepared.ambiguous_join_spaces;
            prepared_text.push(prepared);
        }

        for (row, entry_ref) in translated.entries.iter().zip(refs.iter()) {
            match *entry_ref {
                EntryRef::Text(record_index) => {
                    let record = &self.text_records[record_index];
                    let prepared = &prepared_text[record_index];
                    if !prepared.entry_changed {
                        stats.unchanged += 1;
                        continue;
                    }
                    apply_text_row(
                        &mut body,
                        &mut appended,
                        self.data_size,
                        record,
                        row,
                        prepared,
                        target_encoding,
                        relative_file,
                        &mut stats,
                    )?;
                    stats.patched += 1;
                }
                EntryRef::Choice { .. } => {}
            }
        }

        let resolved_links = self.resolve_links(relative_file)?;
        apply_hyperlinks(
            self,
            &mut body,
            &mut appended,
            &resolved_links,
            &prepared_text,
            relative_file,
            &mut stats,
        )?;

        for (group_index, group) in self.choice_groups.iter().enumerate() {
            let rows = choice_rows.get(&group_index).cloned().unwrap_or_default();
            if rows.len() != group.options.len() {
                return Err(format!(
                    "{relative_file}: choice instruction 0x{:X} has {} JSON rows, expected {}",
                    group.instruction.code_offset,
                    rows.len(),
                    group.options.len()
                ));
            }
            let changed = rows
                .iter()
                .enumerate()
                .any(|(index, row)| row.message != group.options[index].text);
            if !changed {
                stats.unchanged += rows.len() as u64;
                continue;
            }
            let mut block = group.prefix.clone();
            for (option_index, row) in rows.iter().enumerate() {
                if row.message.contains('\r') || row.message.contains('\n') {
                    return Err(format!(
                        "{relative_file}: choice {} at instruction 0x{:X} cannot contain a newline",
                        option_index, group.instruction.code_offset
                    ));
                }
                let encoded = target_encoding.encode(
                    &row.message,
                    &format!(
                        "{relative_file}: choice {} at instruction 0x{:X}",
                        option_index, group.instruction.code_offset
                    ),
                )?;
                append_len_string(&mut block, &encoded, relative_file)?;
            }
            let operand = append_data(&mut appended, self.data_size, &block, relative_file)?;
            put_u32(
                &mut body,
                group.instruction.absolute_offset + 4,
                operand,
                relative_file,
            )?;
            stats.patched += rows.len() as u64;
        }

        if stats.patched == 0 {
            return Ok((self.bytes.clone(), stats));
        }
        body.extend_from_slice(&appended);
        let new_data_size = self
            .data_size
            .checked_add(appended.len())
            .ok_or_else(|| format!("{relative_file}: data_size overflow"))?;
        put_u32(
            &mut body,
            300,
            u32::try_from(new_data_size)
                .map_err(|_| format!("{relative_file}: data_size exceeds u32"))?,
            relative_file,
        )?;
        let digest = md5(&body);
        body.extend_from_slice(&digest);
        Ok((body, stats))
    }
}

fn prepare_text_row(
    record: &TextRecord,
    expected: &TranslationEntry,
    translated: &TranslationEntry,
    relative_file: &str,
) -> ToolResult<PreparedTextRow> {
    let context = format!(
        "{relative_file}: entry {} instruction 0x{:X}",
        translated.index, record.instruction.code_offset
    );
    let expected_markup = parse_link_markup(&expected.scr_msg, &format!("{context}: scr_msg"))?;
    let translated_markup = parse_link_markup(&translated.message, &format!("{context}: message"))?;
    let (translated_markup, removed_join_spaces, ambiguous_join_spaces) =
        if translated.message != expected.message {
            let (_, body_parts) = split_name_body(record, &context)?;
            clean_joined_fullwidth_spaces(body_parts, translated_markup, &context)?
        } else {
            (translated_markup, 0, 0)
        };
    let expected_ids = expected_markup
        .links
        .keys()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    for id in translated_markup.links.keys() {
        if !expected_ids.contains(id) {
            return Err(format!(
                "{context}: hyperlink id {id} does not exist in the source"
            ));
        }
    }
    if translated_markup.plain.contains('\r') {
        return Err(format!(
            "{context}: CR is not allowed; use LF for a hard break"
        ));
    }
    if translated_markup.plain.contains('\0') {
        return Err(format!("{context}: message cannot contain NUL"));
    }

    let name_changed = translated.name != expected.name;
    let text_changed = translated_markup.plain != expected_markup.plain;
    Ok(PreparedTextRow {
        json_index: translated.index,
        markup: translated_markup,
        entry_changed: translated.message != expected.message || name_changed,
        text_changed,
        rebuilt: name_changed || text_changed,
        removed_join_spaces,
        ambiguous_join_spaces,
    })
}

fn apply_hyperlinks(
    scenario: &Scenario,
    body: &mut [u8],
    appended: &mut Vec<u8>,
    resolved_links: &[Vec<ResolvedLink>],
    prepared_text: &[PreparedTextRow],
    relative_file: &str,
    stats: &mut ApplyStats,
) -> ToolResult<()> {
    for (target_record, links) in resolved_links.iter().enumerate() {
        for resolved in links {
            if resolved.target_record != target_record || resolved.anchor_record > target_record {
                return Err(format!(
                    "{relative_file}: internal hyperlink assignment is inconsistent"
                ));
            }
            let translated_span = prepared_text[target_record]
                .markup
                .links
                .get(&resolved.id)
                .copied();
            let group_rebuilt = prepared_text[resolved.anchor_record..=target_record]
                .iter()
                .any(|prepared| prepared.rebuilt);
            let range_changed =
                translated_span != Some((resolved.source_start, resolved.source_len));
            if !group_rebuilt && !range_changed {
                continue;
            }

            let anchor = &scenario.text_records[resolved.anchor_record];
            let link = &anchor.links[resolved.link_index];
            let target = &scenario.text_records[target_record];
            let context = format!(
                "{relative_file}: entry {} instruction 0x{:X}: hyperlink {}",
                prepared_text[target_record].json_index,
                target.instruction.code_offset,
                resolved.id
            );

            let Some((start, len)) = translated_span else {
                neutralize_instruction(body, link.instruction, &context)?;
                stats.warnings.push(format!(
                    "{context} was disabled because its inline tag was removed"
                ));
                continue;
            };

            let (local_line, first, link_len) = if prepared_text[target_record].rebuilt {
                absolute_to_line_col(
                    &prepared_text[target_record].markup.plain,
                    start,
                    len,
                    &context,
                )?
            } else {
                let (_, body_parts) = split_name_body(target, &context)?;
                absolute_to_body_part_line_col(body_parts, start, len, &context)?
            };

            let mut prior_lines = 0usize;
            for (record, prepared) in scenario.text_records[resolved.anchor_record..target_record]
                .iter()
                .zip(&prepared_text[resolved.anchor_record..target_record])
            {
                prior_lines = prior_lines
                    .checked_add(output_body_line_count(record, prepared, &context)?)
                    .ok_or_else(|| format!("{context}: display-group line count overflow"))?;
            }
            let group_line = prior_lines
                .checked_add(local_line as usize)
                .ok_or_else(|| format!("{context}: display-group line count overflow"))?;
            let group_line = u16::try_from(group_line)
                .map_err(|_| format!("{context}: display-group line exceeds u16"))?;

            let mut block = Vec::with_capacity(18);
            block.extend_from_slice(&group_line.to_le_bytes());
            block.extend_from_slice(&first.to_le_bytes());
            block.extend_from_slice(&link_len.to_le_bytes());
            block.extend_from_slice(&link.variable.to_le_bytes());
            block.extend_from_slice(&link.target_entry.to_le_bytes());
            block.extend_from_slice(&link.trailing.to_le_bytes());
            let operand = append_data(appended, scenario.data_size, &block, &context)?;
            put_u32(
                body,
                link.instruction.absolute_offset + 4,
                operand,
                &context,
            )?;
        }
    }
    Ok(())
}

fn output_body_line_count(
    record: &TextRecord,
    prepared: &PreparedTextRow,
    context: &str,
) -> ToolResult<usize> {
    if prepared.rebuilt {
        Ok(prepared.markup.plain.split('\n').count())
    } else {
        let (_, body_parts) = split_name_body(record, context)?;
        Ok(body_parts.len())
    }
}

fn absolute_to_body_part_line_col(
    body_parts: &[String],
    start: usize,
    len: usize,
    context: &str,
) -> ToolResult<(u16, u16, u16)> {
    if len == 0 {
        return Err(format!("{context}: hyperlink has an empty range"));
    }
    let end = start
        .checked_add(len)
        .ok_or_else(|| format!("{context}: hyperlink range overflow"))?;
    let mut cursor = 0usize;
    for (line_index, part) in body_parts.iter().enumerate() {
        let part_end = cursor
            .checked_add(part.chars().count())
            .ok_or_else(|| format!("{context}: text length overflow"))?;
        if start >= cursor && start < part_end {
            if end > part_end {
                return Err(format!(
                    "{context}: hyperlink cannot cross an original body-slot boundary"
                ));
            }
            let line = u16::try_from(line_index + 1)
                .map_err(|_| format!("{context}: line number exceeds u16"))?;
            let first = u16::try_from(start - cursor + 1)
                .map_err(|_| format!("{context}: column exceeds u16"))?;
            let link_len =
                u16::try_from(len).map_err(|_| format!("{context}: link length exceeds u16"))?;
            return Ok((line, first, link_len));
        }
        cursor = part_end;
    }
    Err(format!(
        "{context}: hyperlink range starts outside the original body slots"
    ))
}

#[allow(clippy::too_many_arguments)]
fn apply_text_row(
    body: &mut [u8],
    appended: &mut Vec<u8>,
    original_data_size: usize,
    record: &TextRecord,
    translated: &TranslationEntry,
    prepared: &PreparedTextRow,
    target_encoding: TextEncoding,
    relative_file: &str,
    stats: &mut ApplyStats,
) -> ToolResult<()> {
    let context = format!(
        "{relative_file}: entry {} instruction 0x{:X}",
        translated.index, record.instruction.code_offset
    );
    if prepared.rebuilt {
        if record.ruby_count != 0 {
            return Err(format!(
                "{context}: inline ruby data is present and cannot be relocated safely"
            ));
        }
        let strings =
            translated_text_strings(record, translated, &prepared.markup.plain, &context)?;
        let mut encoded_strings = Vec::with_capacity(strings.len());
        for (index, string) in strings.iter().enumerate() {
            let encoded = target_encoding.encode(string, &format!("{context}: string {index}"))?;
            if encoded.len() > MAX_TEXT_PAYLOAD {
                return Err(format!(
                    "{context}: encoded string {index} is {} bytes; limit is {MAX_TEXT_PAYLOAD}",
                    encoded.len()
                ));
            }
            encoded_strings.push(encoded);
        }
        let block = build_text_block(record, &encoded_strings, &context)?;
        let operand = append_data(appended, original_data_size, &block, &context)?;
        put_u32(
            body,
            record.instruction.absolute_offset + 4,
            operand,
            &context,
        )?;

        if prepared.text_changed {
            for ruby in &record.rubies {
                neutralize_instruction(body, ruby.instruction, &context)?;
                stats.warnings.push(format!(
                    "{context}: ruby instruction 0x{:X} was disabled after text translation",
                    ruby.instruction.code_offset
                ));
            }
            for font in &record.fonts {
                if font.line > 1 {
                    neutralize_instruction(body, font.instruction, &context)?;
                    stats.warnings.push(format!(
                        "{context}: font instruction 0x{:X} targeting original line {} was disabled after line flattening",
                        font.instruction.code_offset, font.line
                    ));
                }
            }
        }
    }
    Ok(())
}

fn clean_joined_fullwidth_spaces(
    body_parts: &[String],
    markup: ParsedMarkup,
    context: &str,
) -> ToolResult<(ParsedMarkup, u64, u64)> {
    let mut source_space_is_join = Vec::new();
    for (part_index, part) in body_parts.iter().enumerate() {
        for (char_index, ch) in part.chars().enumerate() {
            if ch == '\u{3000}' {
                source_space_is_join.push(part_index > 0 && char_index == 0);
            }
        }
    }
    if !source_space_is_join.iter().any(|is_join| *is_join) {
        return Ok((markup, 0, 0));
    }

    let translated_spaces = markup
        .plain
        .chars()
        .enumerate()
        .filter_map(|(position, ch)| (ch == '\u{3000}').then_some(position))
        .collect::<Vec<_>>();
    let source_count = source_space_is_join.len();
    let translated_count = translated_spaces.len();
    if translated_count == 0 {
        return Ok((markup, 0, 0));
    }

    let mut removed_positions = Vec::new();
    let mut ambiguous = 0u64;
    if translated_count <= source_count {
        for (translated_index, &position) in translated_spaces.iter().enumerate() {
            // Treat translated full-width spaces as an order-preserving subsequence of the
            // source spaces. Delete only when every possible source match is a physical
            // slot-leading space; otherwise preserve it rather than risk deleting content.
            let first_source = translated_index;
            let last_source = source_count - translated_count + translated_index;
            let possible = &source_space_is_join[first_source..=last_source];
            if possible.iter().all(|is_join| *is_join) {
                removed_positions.push(position);
            } else if possible.iter().any(|is_join| *is_join) {
                ambiguous += 1;
            }
        }
    } else {
        // The translator added full-width spaces. Their relationship to source slot
        // boundaries is no longer provable, so keep every one.
        ambiguous = source_space_is_join
            .iter()
            .filter(|is_join| **is_join)
            .count() as u64;
    }
    if removed_positions.is_empty() {
        return Ok((markup, 0, ambiguous));
    }

    let mut plain = String::with_capacity(markup.plain.len());
    let mut remove_cursor = 0usize;
    for (position, ch) in markup.plain.chars().enumerate() {
        if removed_positions.get(remove_cursor) == Some(&position) {
            remove_cursor += 1;
        } else {
            plain.push(ch);
        }
    }

    let mut links = BTreeMap::new();
    for (&id, &(start, len)) in &markup.links {
        let end = start
            .checked_add(len)
            .ok_or_else(|| format!("{context}: hyperlink {id} range overflow during cleanup"))?;
        let removed_before_start = removed_positions
            .iter()
            .take_while(|position| **position < start)
            .count();
        let removed_before_end = removed_positions
            .iter()
            .take_while(|position| **position < end)
            .count();
        let new_start = start - removed_before_start;
        let new_end = end - removed_before_end;
        if new_start >= new_end {
            return Err(format!(
                "{context}: hyperlink {id} became empty after joined-space cleanup"
            ));
        }
        links.insert(id, (new_start, new_end - new_start));
    }
    Ok((
        ParsedMarkup { plain, links },
        removed_positions.len() as u64,
        ambiguous,
    ))
}

fn translated_text_strings(
    record: &TextRecord,
    translated: &TranslationEntry,
    plain_message: &str,
    context: &str,
) -> ToolResult<Vec<String>> {
    let mut strings = Vec::new();
    if record.instruction.flags == 0x13 {
        let name = translated
            .name
            .as_ref()
            .ok_or_else(|| format!("{context}: dialogue name is missing"))?;
        if name.contains('\r') || name.contains('\n') || name.contains('\0') {
            return Err(format!("{context}: name cannot contain CR, LF, or NUL"));
        }
        strings.push(name.clone());
    }
    if plain_message.contains('\0') {
        return Err(format!("{context}: message cannot contain NUL"));
    }
    strings.extend(plain_message.split('\n').map(ToOwned::to_owned));
    if strings.is_empty() {
        strings.push(String::new());
    }
    Ok(strings)
}

fn build_text_block(
    record: &TextRecord,
    encoded_strings: &[Vec<u8>],
    context: &str,
) -> ToolResult<Vec<u8>> {
    let mut block = Vec::new();
    block.extend_from_slice(&record.ident.to_le_bytes());
    block.extend_from_slice(&0u32.to_le_bytes());
    block.extend_from_slice(&record.meta.to_le_bytes());
    block.extend_from_slice(
        &u32::try_from(encoded_strings.len())
            .map_err(|_| format!("{context}: too many text strings"))?
            .to_le_bytes(),
    );
    for encoded in encoded_strings {
        append_len_string(&mut block, encoded, context)?;
    }
    let text_size = block
        .len()
        .checked_sub(8)
        .ok_or_else(|| format!("{context}: text block size underflow"))?;
    block[4..8].copy_from_slice(
        &u32::try_from(text_size)
            .map_err(|_| format!("{context}: text section exceeds u32"))?
            .to_le_bytes(),
    );
    block.extend_from_slice(&record.ruby_tail);
    Ok(block)
}

fn append_len_string(block: &mut Vec<u8>, encoded: &[u8], context: &str) -> ToolResult<()> {
    let len = u32::try_from(encoded.len())
        .map_err(|_| format!("{context}: encoded string exceeds u32"))?;
    block.extend_from_slice(&len.to_le_bytes());
    block.extend(encoded.iter().map(|byte| byte ^ 0x53));
    Ok(())
}

fn append_data(
    appended: &mut Vec<u8>,
    original_data_size: usize,
    block: &[u8],
    context: &str,
) -> ToolResult<u32> {
    let offset = original_data_size
        .checked_add(appended.len())
        .ok_or_else(|| format!("{context}: appended data offset overflow"))?;
    let offset = u32::try_from(offset)
        .map_err(|_| format!("{context}: appended data offset exceeds u32"))?;
    appended.extend_from_slice(block);
    Ok(offset)
}

fn neutralize_instruction(
    body: &mut [u8],
    instruction: Instruction,
    context: &str,
) -> ToolResult<()> {
    let target = body
        .get_mut(instruction.absolute_offset..instruction.absolute_offset + INSTRUCTION_SIZE)
        .ok_or_else(|| {
            format!(
                "{context}: instruction 0x{:X} is outside output body",
                instruction.code_offset
            )
        })?;
    target.fill(0);
    put_u16(body, instruction.absolute_offset, 0, context)
}

fn validate_translation_file(
    expected: &TranslationFile,
    translated: &TranslationFile,
) -> ToolResult<()> {
    if translated.format != FORMAT_ID {
        return Err(format!(
            "{}: unsupported _format {:?}",
            translated.file, translated.format
        ));
    }
    if translated.file != expected.file
        || translated.scenario_id != expected.scenario_id
        || translated.title != expected.title
        || translated.source_md5 != expected.source_md5
    {
        return Err(format!(
            "{}: top-level source metadata does not match the selected source file",
            translated.file
        ));
    }
    if translated.entries.len() != expected.entries.len() {
        return Err(format!(
            "{}: JSON has {} entries, expected {}",
            translated.file,
            translated.entries.len(),
            expected.entries.len()
        ));
    }
    for (index, (actual, source)) in translated
        .entries
        .iter()
        .zip(expected.entries.iter())
        .enumerate()
    {
        let immutable_matches = actual.file == source.file
            && actual.index == source.index
            && actual.offset == source.offset
            && actual.inst_offset == source.inst_offset
            && actual.size == source.size
            && actual.entry_type == source.entry_type
            && actual.opcode == source.opcode
            && actual.encoding == source.encoding
            && actual.policy == source.policy
            && actual.flags == source.flags
            && actual.choice_index == source.choice_index
            && actual.scr_name == source.scr_name
            && actual.scr_msg == source.scr_msg
            && actual.links == source.links;
        if !immutable_matches {
            return Err(format!(
                "{}: immutable source metadata differs at entry {index}",
                translated.file
            ));
        }
        if source.name.is_some() != actual.name.is_some() {
            return Err(format!(
                "{}: name presence differs at entry {index}",
                translated.file
            ));
        }
        if actual
            .name
            .as_deref()
            .is_some_and(|name| name.contains('\0'))
        {
            return Err(format!("{}: NUL in name at entry {index}", translated.file));
        }
    }
    Ok(())
}

#[cfg(test)]
mod joined_space_tests {
    use super::*;

    #[test]
    fn removes_only_proven_slot_leading_spaces() {
        let parts = vec![
            "　Ghost　本文。".to_string(),
            "　続き。".to_string(),
            "　終わり。".to_string(),
        ];
        let markup = ParsedMarkup {
            plain: "　Ghost　正文。　继续。　结束。".to_string(),
            links: BTreeMap::new(),
        };
        let (cleaned, removed, ambiguous) =
            clean_joined_fullwidth_spaces(&parts, markup, "test").unwrap();
        assert_eq!(cleaned.plain, "　Ghost　正文。继续。结束。");
        assert_eq!(removed, 2);
        assert_eq!(ambiguous, 0);
    }

    #[test]
    fn preserves_ambiguous_internal_space_after_translation_deleted_one() {
        let parts = vec!["　Ghost　本文。".to_string(), "　続き。".to_string()];
        let markup = ParsedMarkup {
            plain: "　Ghost　正文。".to_string(),
            links: BTreeMap::new(),
        };
        let (cleaned, removed, ambiguous) =
            clean_joined_fullwidth_spaces(&parts, markup, "test").unwrap();
        assert_eq!(cleaned.plain, "　Ghost　正文。");
        assert_eq!(removed, 0);
        assert_eq!(ambiguous, 1);
    }

    #[test]
    fn shifts_and_shrinks_hyperlink_ranges() {
        let parts = vec!["　前。".to_string(), "　交換語".to_string()];
        let mut links = BTreeMap::new();
        links.insert(0, (3, 4));
        let markup = ParsedMarkup {
            plain: "　前。　交換語".to_string(),
            links,
        };
        let (cleaned, removed, ambiguous) =
            clean_joined_fullwidth_spaces(&parts, markup, "test").unwrap();
        assert_eq!(cleaned.plain, "　前。交換語");
        assert_eq!(cleaned.links.get(&0), Some(&(3, 3)));
        assert_eq!(removed, 1);
        assert_eq!(ambiguous, 0);
    }

    fn test_instruction(code_offset: usize, opcode: u16) -> Instruction {
        Instruction {
            absolute_offset: code_offset,
            code_offset,
            opcode,
            flags: 0x12,
            operand: 0,
        }
    }

    fn test_record(code_offset: usize, parts: &[&str], links: Vec<LinkControl>) -> TextRecord {
        TextRecord {
            instruction: test_instruction(code_offset, 0x40),
            record_offset: 0,
            record_size: 0,
            ident: 0,
            meta: 0,
            parts: parts.iter().map(|part| (*part).to_string()).collect(),
            ruby_tail: Vec::new(),
            ruby_count: 0,
            links,
            rubies: Vec::new(),
            fonts: Vec::new(),
        }
    }

    fn continuation_scenario(separator_opcode: u16) -> Scenario {
        let link_instruction = test_instruction(0, 0x46);
        let link = LinkControl {
            instruction: link_instruction,
            line: 3,
            first: 2,
            len: 2,
            variable: 0,
            target_entry: 0,
            trailing: 0,
            target: ScriptEntry {
                scenario_id: 1,
                entry_index: 0,
                code_offset: 0,
            },
        };
        Scenario {
            bytes: Vec::new(),
            scenario_id: 1,
            title: String::new(),
            entries: Vec::new(),
            instructions: vec![
                link_instruction,
                test_instruction(12, 0x40),
                test_instruction(24, separator_opcode),
                test_instruction(36, 0x40),
                test_instruction(48, 0x53),
            ],
            text_records: vec![
                test_record(12, &["甲", "乙"], vec![link]),
                test_record(36, &["丙交換丁"], Vec::new()),
            ],
            choice_groups: Vec::new(),
            code_start: 0,
            code_size: 60,
            data_start: 60,
            data_size: 0,
            body_end: 60,
        }
    }

    #[test]
    fn resolves_hyperlink_across_continued_text_records() {
        let scenario = continuation_scenario(0x52);
        let resolved = scenario.resolve_links("test").unwrap();
        assert!(resolved[0].is_empty());
        assert_eq!(resolved[1].len(), 1);
        let link = &resolved[1][0];
        assert_eq!(link.anchor_record, 0);
        assert_eq!(link.target_record, 1);
        assert_eq!((link.source_start, link.source_len), (1, 2));
    }

    #[test]
    fn rejects_hyperlink_crossing_display_group_terminator() {
        let scenario = continuation_scenario(0x53);
        let error = scenario.resolve_links("test").unwrap_err();
        assert!(error.contains("does not resolve before the display-group terminator"));
    }

    #[test]
    fn maps_flat_range_back_to_original_body_slot() {
        let parts = vec!["甲乙".to_string(), "丙交換丁".to_string()];
        let coordinates = absolute_to_body_part_line_col(&parts, 3, 2, "test").unwrap();
        assert_eq!(coordinates, (2, 2, 2));
    }
}
