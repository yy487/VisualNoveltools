//! Structured MJO text extraction for the cleaned localization path.
//!
//! This module groups only text operands joined by explicit VM events. Newline
//! and ruby instructions live in `NativeEvent` metadata and never enter the
//! translator-facing message. Their source instructions are removed on import.

use std::collections::{BTreeMap, BTreeSet};

use super::mjo_bytecode::{reachable_instruction_indices, BytecodeProgram, Instruction};

const RUBY_HELPER_HASH: u32 = 0x3198_FD01;
const AUDIO_HELPER_HASH: u32 = 0x812A_FDF0;
const MENU_HELPER_HASH: u32 = 0x0A7A_489C;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecordKind {
    Display,
    ChoiceOption,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NativeEventKind {
    LineBreak,
    Ruby,
    Audio,
    Dynamic,
    OpaqueParameter,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TextFragment {
    pub(super) offset: usize,
    pub(super) opcode: u16,
    /// Original decoded CP932 text, including any literal CR/LF characters.
    pub(super) source_text: String,
    /// Text projected into the translator message after removing CR/LF.
    pub(super) display_text: String,
    /// Exact native operand bytes, including terminal NUL, as hex.
    pub(super) source_bytes_hex: String,
    /// Unicode-scalar positions of physical CR/LF characters in `source_text`.
    pub(super) physical_newline_positions: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct NativeEvent {
    /// Fragment boundary at which this event appears in the source program.
    pub(super) before_fragment_index: usize,
    pub(super) offset: usize,
    pub(super) opcode: u16,
    pub(super) kind: NativeEventKind,
    pub(super) source_text: Option<String>,
    pub(super) source_bytes_hex: Option<String>,
    /// For ruby, the immediately following 2112 operand is the target text.
    pub(super) related_offset: Option<usize>,
    /// Only formatting values needed in translator text are visible tokens.
    pub(super) visible_token: Option<String>,
}

struct Bridge {
    events: Vec<NativeEvent>,
    crossed_display: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ChoiceFlowEdge {
    pub(super) instruction_offset: usize,
    pub(super) opcode: u16,
    pub(super) target_offset: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ChoiceControl {
    pub(super) call_offset: usize,
    pub(super) callee_hash: u32,
    pub(super) argument_count: u16,
    /// Index in visible menu order, which reverses the 2049 push sequence.
    pub(super) option_index: u16,
    pub(super) source_option_offsets: Vec<usize>,
    pub(super) flow_edges: Vec<ChoiceFlowEdge>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ScriptTextRecord {
    pub(super) kind: RecordKind,
    /// First source operand offset. Choice options retain their own 2049 offset.
    pub(super) offset: usize,
    pub(super) opcode: u16,
    pub(super) name: Option<String>,
    pub(super) speaker_quote_closed: bool,
    pub(super) choice_quote_wrapped: bool,
    pub(super) choice_control: Option<ChoiceControl>,
    pub(super) fragments: Vec<TextFragment>,
    pub(super) events: Vec<NativeEvent>,
    pub(super) message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ExtractionReport {
    pub(super) records: Vec<ScriptTextRecord>,
    pub(super) dynamic_instruction_count: usize,
    pub(super) dynamic_append_instruction_count: usize,
    pub(super) skipped_dynamic_offsets: Vec<usize>,
    pub(super) skipped_unreachable_offsets: Vec<usize>,
    pub(super) skipped_blank_offsets: Vec<usize>,
    pub(super) removed_instruction_offsets: BTreeSet<usize>,
    pub(super) removed_linebreak_count: usize,
    pub(super) removed_ruby_count: usize,
    pub(super) choice_group_count: usize,
    pub(super) choice_option_count: usize,
    pub(super) native_event_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct TextEdits {
    pub(super) replacement_operands: BTreeMap<usize, Vec<u8>>,
    pub(super) removed_instruction_offsets: BTreeSet<usize>,
}

pub(super) fn extract_records_with_report(
    _payload: &[u8],
    program: &BytecodeProgram,
) -> Result<ExtractionReport, String> {
    let reachable = reachable_instruction_indices(program);
    let mut report = ExtractionReport {
        dynamic_instruction_count: program
            .instructions
            .iter()
            .filter(|instruction| instruction.opcode == 2114)
            .count(),
        dynamic_append_instruction_count: program
            .instructions
            .iter()
            .filter(|instruction| {
                instruction.opcode == 2114
                    && instruction
                        .text_bytes
                        .as_deref()
                        .is_some_and(|bytes| bytes.first() == Some(&b'd'))
            })
            .count(),
        ..ExtractionReport::default()
    };
    report.removed_instruction_offsets = control_instructions_to_remove(program);
    report.removed_linebreak_count = program
        .instructions
        .iter()
        .filter(|instruction| is_linebreak(instruction))
        .count();
    report.removed_ruby_count = report
        .removed_instruction_offsets
        .iter()
        .filter(|offset| {
            program.instructions.iter().any(|instruction| {
                instruction.offset == **offset && matches!(instruction.opcode, 2049 | 2064)
            })
        })
        .count()
        / 2;

    let dynamic_skipped = find_dynamic_regions(program);
    let mut records = Vec::new();
    for (instruction_index, instruction) in program.instructions.iter().enumerate() {
        if instruction.opcode != 2112 {
            continue;
        }
        if !reachable.contains(&instruction_index) {
            report.skipped_unreachable_offsets.push(instruction.offset);
            continue;
        }
        let operand = decode_operand(instruction)?;
        if operand.source_text.chars().all(char::is_whitespace) {
            report.skipped_blank_offsets.push(instruction.offset);
            continue;
        }
        let mut record = ScriptTextRecord {
            kind: RecordKind::Display,
            offset: instruction.offset,
            opcode: 2112,
            name: None,
            speaker_quote_closed: false,
            choice_quote_wrapped: false,
            choice_control: None,
            fragments: vec![operand],
            events: Vec::new(),
            message: String::new(),
        };
        refresh_translation_fields(&mut record);
        records.push(record);
    }

    records = merge_structurally_linked_records(records, program)?;
    let represented_dynamic = records
        .iter()
        .filter(|record| {
            record
                .events
                .iter()
                .any(|event| event.kind == NativeEventKind::Dynamic)
        })
        .flat_map(|record| record.fragments.iter().map(|fragment| fragment.offset))
        .collect::<BTreeSet<_>>();
    for (index, instruction) in program.instructions.iter().enumerate() {
        if instruction.opcode == 2112
            && reachable.contains(&index)
            && dynamic_skipped.contains(&instruction.offset)
            && !represented_dynamic.contains(&instruction.offset)
        {
            report.skipped_dynamic_offsets.push(instruction.offset);
        }
    }
    records.retain(|record| {
        !record
            .fragments
            .iter()
            .any(|fragment| report.skipped_dynamic_offsets.contains(&fragment.offset))
    });

    let (choice_records, choice_group_count) = extract_choice_records(program, &reachable)?;
    report.choice_group_count = choice_group_count;
    report.choice_option_count = choice_records.len();
    records.extend(choice_records);
    for record in &mut records {
        refresh_translation_fields(record);
    }
    records.sort_by_key(|record| match &record.choice_control {
        Some(control) => (control.call_offset, 1u8, control.option_index),
        None => (record.offset, 0u8, 0u16),
    });
    report.native_event_count = records.iter().map(|record| record.events.len()).sum();
    report.records = records;
    report.skipped_dynamic_offsets.sort_unstable();
    report.skipped_unreachable_offsets.sort_unstable();
    report.skipped_blank_offsets.sort_unstable();
    Ok(report)
}

fn merge_structurally_linked_records(
    records: Vec<ScriptTextRecord>,
    program: &BytecodeProgram,
) -> Result<Vec<ScriptTextRecord>, String> {
    let mut merged = Vec::with_capacity(records.len());
    let mut index = 0usize;
    while index < records.len() {
        let mut current = records[index].clone();
        let mut consumed = 1usize;
        loop {
            let Some(next) = records.get(index + consumed) else {
                break;
            };
            if next.kind != RecordKind::Display || next.name.is_some() {
                break;
            }
            let Some(previous_fragment) = current.fragments.last() else {
                break;
            };
            let Some(mut bridge) = events_between(program, previous_fragment.offset, next.offset)?
            else {
                break;
            };
            let mut events = std::mem::take(&mut bridge.events);
            let open_speaker = current.name.is_some() && !current.speaker_quote_closed;
            let physical_newline = previous_fragment.source_text.ends_with(['\r', '\n'])
                || next.fragments[0].source_text.starts_with(['\r', '\n']);
            let explicit_link = events.iter().any(|event| {
                matches!(
                    event.kind,
                    NativeEventKind::LineBreak | NativeEventKind::Ruby | NativeEventKind::Dynamic
                )
            });
            if !open_speaker && !physical_newline && !explicit_link && bridge.crossed_display {
                let boundary = current.fragments.len();
                for event in &mut events {
                    event.before_fragment_index = boundary;
                }
                current.events.extend(events);
                refresh_translation_fields(&mut current);
                break;
            }
            let boundary = current.fragments.len();
            for event in &mut events {
                event.before_fragment_index = boundary;
            }
            current.events.extend(events);
            current.fragments.push(next.fragments[0].clone());
            refresh_translation_fields(&mut current);
            consumed += 1;
            if current.speaker_quote_closed {
                break;
            }
        }
        merged.push(current);
        index += consumed;
    }
    Ok(merged)
}

fn events_between(
    program: &BytecodeProgram,
    first_offset: usize,
    second_offset: usize,
) -> Result<Option<Bridge>, String> {
    let Ok(first) = program
        .instructions
        .binary_search_by_key(&first_offset, |instruction| instruction.offset)
    else {
        return Ok(None);
    };
    let Ok(second) = program
        .instructions
        .binary_search_by_key(&second_offset, |instruction| instruction.offset)
    else {
        return Ok(None);
    };
    if first >= second
        || function_for_instruction(program, first) != function_for_instruction(program, second)
    {
        return Ok(None);
    }
    let mut events = Vec::new();
    let mut crossed_display = false;
    let mut index = first + 1;
    while index < second {
        let instruction = &program.instructions[index];
        match instruction.opcode {
            2113 => crossed_display = true,
            2106 => {}
            2114 => {
                let bytes = instruction
                    .text_bytes
                    .as_deref()
                    .ok_or_else(|| format!("2114 at 0x{:X} has no operand", instruction.offset))?;
                if bytes == b"n\0" {
                    events.push(native_event(
                        instruction,
                        NativeEventKind::LineBreak,
                        Some("n".into()),
                        None,
                        None,
                    )?);
                } else if bytes.first() == Some(&b'd') {
                    events.push(native_event(
                        instruction,
                        NativeEventKind::Dynamic,
                        decode_cp932_z(bytes),
                        None,
                        Some("\\d".into()),
                    )?);
                } else {
                    return Ok(None);
                }
            }
            2049 => {
                let operand = instruction
                    .text_bytes
                    .as_deref()
                    .ok_or_else(|| format!("2049 at 0x{:X} has no operand", instruction.offset))?;
                let text = decode_cp932_z(operand).ok_or_else(|| {
                    format!("2049 at 0x{:X} is invalid CP932", instruction.offset)
                })?;
                let helper = program.instructions.get(index + 1);
                if helper.is_some_and(|helper| {
                    helper.opcode == 2064
                        && helper.script_call_hash == Some(RUBY_HELPER_HASH)
                        && helper.script_call_argument_count == Some(1)
                        && index + 2 == second
                }) {
                    let helper = &program.instructions[index + 1];
                    events.push(native_event(
                        instruction,
                        NativeEventKind::Ruby,
                        Some(text),
                        Some(second_offset),
                        None,
                    )?);
                    index += 1; // Consume the helper call with its reading operand.
                    let _ = helper;
                } else if helper.is_some_and(|helper| {
                    helper.opcode == 2064
                        && helper.script_call_hash == Some(AUDIO_HELPER_HASH)
                        && helper.script_call_argument_count == Some(1)
                        && text.starts_with('v')
                }) {
                    events.push(native_event(
                        instruction,
                        NativeEventKind::Audio,
                        Some(text),
                        None,
                        None,
                    )?);
                    index += 1;
                } else {
                    events.push(native_event(
                        instruction,
                        NativeEventKind::OpaqueParameter,
                        Some(text),
                        None,
                        None,
                    )?);
                }
            }
            _ => return Ok(None),
        }
        index += 1;
    }
    Ok(Some(Bridge {
        events,
        crossed_display,
    }))
}

fn native_event(
    instruction: &Instruction,
    kind: NativeEventKind,
    source_text: Option<String>,
    related_offset: Option<usize>,
    visible_token: Option<String>,
) -> Result<NativeEvent, String> {
    let source_bytes = instruction.text_bytes.as_deref().ok_or_else(|| {
        format!(
            "opcode {} at 0x{:X} has no string operand",
            instruction.opcode, instruction.offset
        )
    })?;
    Ok(NativeEvent {
        before_fragment_index: 0,
        offset: instruction.offset,
        opcode: instruction.opcode,
        kind,
        source_text,
        source_bytes_hex: Some(bytes_to_hex(source_bytes)),
        related_offset,
        visible_token,
    })
}

fn function_for_instruction(program: &BytecodeProgram, index: usize) -> Option<usize> {
    program.functions.iter().position(|function| {
        index >= function.first_instruction && index < function.instruction_end
    })
}

fn extract_choice_records(
    program: &BytecodeProgram,
    reachable: &BTreeSet<usize>,
) -> Result<(Vec<ScriptTextRecord>, usize), String> {
    let mut records = Vec::new();
    let mut group_count = 0usize;
    for (call_index, call) in program.instructions.iter().enumerate() {
        if call.opcode != 2063 || call.script_call_hash != Some(MENU_HELPER_HASH) {
            continue;
        }
        if !reachable.contains(&call_index) {
            continue;
        }
        let count = usize::from(
            call.script_call_argument_count
                .ok_or_else(|| format!("menu helper at 0x{:X} has no parsed argc", call.offset))?,
        );
        if count < 2 {
            return Err(format!(
                "menu helper at 0x{:X} has argc {count}",
                call.offset
            ));
        }
        let first_option = call_index
            .checked_sub(count)
            .ok_or_else(|| format!("menu helper at 0x{:X} lacks option operands", call.offset))?;
        if function_for_instruction(program, first_option)
            != function_for_instruction(program, call_index)
        {
            return Err(format!(
                "menu helper at 0x{:X} crosses a function boundary",
                call.offset
            ));
        }
        let source_args = &program.instructions[first_option..call_index];
        if source_args.len() != count
            || source_args
                .iter()
                .any(|instruction| instruction.opcode != 2049)
        {
            return Err(format!(
                "menu helper at 0x{:X} is not preceded by argc static 2049 labels",
                call.offset
            ));
        }
        if source_args.iter().any(|instruction| {
            program
                .instructions
                .binary_search_by_key(&instruction.offset, |candidate| candidate.offset)
                .ok()
                .is_none_or(|index| !reachable.contains(&index))
        }) {
            return Err(format!(
                "menu helper at 0x{:X} has an unreachable option",
                call.offset
            ));
        }
        group_count += 1;
        let source_option_offsets = source_args
            .iter()
            .map(|instruction| instruction.offset)
            .collect::<Vec<_>>();
        let flow_edges = choice_flow_edges(program, call_index);
        for (visible_index, instruction) in source_args.iter().rev().enumerate() {
            let operand = instruction
                .text_bytes
                .as_deref()
                .ok_or_else(|| format!("choice 2049 at 0x{:X} lacks bytes", instruction.offset))?;
            let source_text = decode_cp932_z(operand).ok_or_else(|| {
                format!("choice 2049 at 0x{:X} is invalid CP932", instruction.offset)
            })?;
            if source_text.chars().all(char::is_whitespace) {
                return Err(format!(
                    "choice 2049 at 0x{:X} is blank",
                    instruction.offset
                ));
            }
            let wrapped = source_text.starts_with('「') && source_text.ends_with('」');
            let display_text = if wrapped {
                source_text
                    .strip_prefix('「')
                    .and_then(|s| s.strip_suffix('」'))
                    .unwrap_or(&source_text)
                    .to_owned()
            } else {
                source_text.clone()
            };
            let display_text = strip_newlines(&display_text);
            records.push(ScriptTextRecord {
                kind: RecordKind::ChoiceOption,
                offset: instruction.offset,
                opcode: 2049,
                name: None,
                speaker_quote_closed: false,
                choice_quote_wrapped: wrapped,
                choice_control: Some(ChoiceControl {
                    call_offset: call.offset,
                    callee_hash: MENU_HELPER_HASH,
                    argument_count: count as u16,
                    option_index: visible_index as u16,
                    source_option_offsets: source_option_offsets.clone(),
                    flow_edges: flow_edges.clone(),
                }),
                fragments: vec![TextFragment {
                    offset: instruction.offset,
                    opcode: 2049,
                    source_text,
                    display_text: display_text.clone(),
                    source_bytes_hex: bytes_to_hex(operand),
                    physical_newline_positions: Vec::new(),
                }],
                events: Vec::new(),
                message: display_text,
            });
        }
    }
    Ok((records, group_count))
}

fn choice_flow_edges(program: &BytecodeProgram, call_index: usize) -> Vec<ChoiceFlowEdge> {
    let Some(function) = program.functions.iter().find(|function| {
        call_index >= function.first_instruction && call_index < function.instruction_end
    }) else {
        return Vec::new();
    };
    let first = call_index + 1;
    let end = (first..function.instruction_end)
        .find(|index| matches!(program.instructions[*index].opcode, 2112 | 2091))
        .unwrap_or(function.instruction_end);
    let offsets = program.instructions[first..end]
        .iter()
        .map(|instruction| instruction.offset)
        .collect::<BTreeSet<_>>();
    program
        .relocations
        .iter()
        .filter(|relocation| offsets.contains(&relocation.instruction_offset))
        .filter_map(|relocation| {
            let index = program
                .instructions
                .binary_search_by_key(&relocation.instruction_offset, |instruction| {
                    instruction.offset
                })
                .ok()?;
            Some(ChoiceFlowEdge {
                instruction_offset: relocation.instruction_offset,
                opcode: program.instructions[index].opcode,
                target_offset: relocation.target_offset,
            })
        })
        .collect()
}

pub(super) fn translation_entries(records: &[ScriptTextRecord]) -> Vec<vn_text::Entry> {
    records
        .iter()
        .map(|record| vn_text::Entry {
            name: record.name.clone(),
            message: record.message.clone(),
        })
        .collect()
}

pub(super) fn primary_source_text(record: &ScriptTextRecord) -> String {
    let mut source = String::new();
    let mut event_index = 0usize;
    for fragment_index in 0..=record.fragments.len() {
        while record
            .events
            .get(event_index)
            .is_some_and(|event| event.before_fragment_index == fragment_index)
        {
            if record.events[event_index].kind == NativeEventKind::LineBreak {
                source.push('\n');
            }
            event_index += 1;
        }
        if let Some(fragment) = record.fragments.get(fragment_index) {
            source.push_str(&fragment.source_text);
        }
    }
    source
}

pub(super) fn translated_text(
    record: &ScriptTextRecord,
    entry: &vn_text::Entry,
) -> Result<String, String> {
    if record.name.is_some() != entry.name.is_some() {
        return Err("translation changed whether this entry has a speaker name".into());
    }
    if record.kind == RecordKind::ChoiceOption {
        if entry.name.is_some() {
            return Err("choice option cannot have a speaker name".into());
        }
        let message = strip_newlines(&entry.message);
        return Ok(if record.choice_quote_wrapped {
            format!("「{message}」")
        } else {
            message
        });
    }
    let message = strip_newlines(&entry.message);
    let composed = match &entry.name {
        Some(name) => {
            if !message.starts_with('「') {
                return Err("message must retain the source speaker quote opener '「'".into());
            }
            if record.speaker_quote_closed && !message.ends_with('」') {
                return Err("message must retain the source speaker quote closer '」'".into());
            }
            format!("{name}{message}")
        }
        None => message,
    };
    Ok(composed)
}

pub(super) fn encoding_projection(record: &ScriptTextRecord, text: &str) -> String {
    record
        .events
        .iter()
        .filter_map(|event| event.visible_token.as_deref())
        .fold(text.to_owned(), |text, token| text.replace(token, ""))
}

pub(super) fn build_edits(
    payload: &[u8],
    program: &BytecodeProgram,
    report: &ExtractionReport,
    entries: &[vn_text::Entry],
    encode: impl Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<TextEdits, String> {
    let expected = translation_entries(&report.records);
    vn_text::validate_shape(&expected, entries)
        .map_err(|error| format!("translation template shape mismatch: {error}"))?;
    let mut edits = TextEdits {
        replacement_operands: BTreeMap::new(),
        removed_instruction_offsets: report.removed_instruction_offsets.clone(),
    };
    for instruction in &program.instructions {
        if instruction.opcode == 2112 {
            let bytes = instruction.text_bytes.as_deref().ok_or_else(|| {
                format!("2112 at 0x{:X} has no parsed operand", instruction.offset)
            })?;
            let sanitized = strip_newline_bytes(bytes);
            edits
                .replacement_operands
                .insert(instruction.offset, sanitized);
        }
    }
    for (record, entry) in report.records.iter().zip(entries) {
        let translated = translated_text(record, entry)?;
        let pieces = distribute_translation(record, entry, &translated)?;
        for (offset, text) in pieces {
            let encoded = encode(&strip_newlines(&text))?;
            if encoded.contains(&0) {
                return Err(format!(
                    "translation at source offset 0x{offset:X} contains NUL"
                ));
            }
            let total_len = encoded
                .len()
                .checked_add(1)
                .ok_or_else(|| format!("translation at 0x{offset:X} length overflows"))?;
            if total_len > u16::MAX as usize {
                return Err(format!(
                    "translation at 0x{offset:X} exceeds u16 byte length"
                ));
            }
            let mut operand = encoded;
            operand.push(0);
            edits.replacement_operands.insert(offset, operand);
        }
    }
    for offset in &edits.removed_instruction_offsets {
        if edits.replacement_operands.contains_key(offset) {
            return Err(format!(
                "removed instruction 0x{offset:X} also has a text replacement"
            ));
        }
    }
    let _ = payload;
    Ok(edits)
}

fn distribute_translation(
    record: &ScriptTextRecord,
    entry: &vn_text::Entry,
    translated: &str,
) -> Result<Vec<(usize, String)>, String> {
    let visible_events = record
        .events
        .iter()
        .filter(|event| event.visible_token.is_some())
        .collect::<Vec<_>>();
    if visible_events.is_empty() {
        return distribute_to_fragment_range(record, entry, 0, record.fragments.len(), translated);
    }
    let mut pieces = Vec::new();
    let mut fragment_start = 0usize;
    let mut event_index = 0usize;
    let mut remaining = translated;
    while event_index < visible_events.len() {
        let boundary = visible_events[event_index].before_fragment_index;
        if boundary < fragment_start || boundary > record.fragments.len() {
            return Err("visible event boundary exceeds text fragment range".into());
        }
        let mut group_end = event_index + 1;
        while group_end < visible_events.len()
            && visible_events[group_end].before_fragment_index == boundary
        {
            group_end += 1;
        }
        let token = visible_events[event_index]
            .visible_token
            .as_deref()
            .unwrap_or_default();
        let Some(token_position) = remaining.find(token) else {
            return Err(format!("missing protected token {token}"));
        };
        pieces.extend(distribute_to_fragment_range(
            record,
            entry,
            fragment_start,
            boundary,
            &remaining[..token_position],
        )?);
        remaining = &remaining[token_position..];
        for event in &visible_events[event_index..group_end] {
            let token = event.visible_token.as_deref().unwrap_or_default();
            if !remaining.starts_with(token) {
                return Err(format!(
                    "protected token order mismatch at boundary {boundary}"
                ));
            }
            remaining = &remaining[token.len()..];
        }
        fragment_start = boundary;
        event_index = group_end;
    }
    pieces.extend(distribute_to_fragment_range(
        record,
        entry,
        fragment_start,
        record.fragments.len(),
        remaining,
    )?);
    Ok(pieces)
}

fn distribute_to_fragment_range(
    record: &ScriptTextRecord,
    entry: &vn_text::Entry,
    start: usize,
    end: usize,
    translated: &str,
) -> Result<Vec<(usize, String)>, String> {
    if start > end || end > record.fragments.len() {
        return Err("text fragment range is invalid".into());
    }
    let fragments = &record.fragments[start..end];
    if fragments.is_empty() {
        return if translated.is_empty() {
            Ok(Vec::new())
        } else {
            Err("cannot distribute text between adjacent protected events".into())
        };
    }
    if fragments.len() == 1 {
        return Ok(vec![(fragments[0].offset, translated.to_owned())]);
    }
    let speaker_prefix_len = if start == 0 {
        record
            .name
            .as_deref()
            .map_or(0, |name| name.chars().count())
    } else {
        0
    };
    let original_body = fragments
        .iter()
        .map(|fragment| fragment.display_text.chars().count())
        .sum::<usize>()
        .saturating_sub(speaker_prefix_len);
    let translated_chars = translated.chars().collect::<Vec<_>>();
    let translated_body = translated_chars.len().saturating_sub(speaker_prefix_len);
    let mut result = Vec::with_capacity(fragments.len());
    let mut original_boundary = 0usize;
    let mut new_boundary = 0usize;
    for (index, fragment) in fragments.iter().enumerate() {
        if index + 1 == fragments.len() {
            result.push((
                fragment.offset,
                translated_chars[new_boundary..].iter().collect(),
            ));
            break;
        }
        original_boundary += fragment.display_text.chars().count();
        let old_body_boundary = original_boundary.saturating_sub(speaker_prefix_len);
        let projected = if original_body == 0 {
            0
        } else {
            usize::try_from(
                (old_body_boundary as u128 * translated_body as u128 + original_body as u128 / 2)
                    / original_body as u128,
            )
            .unwrap_or(translated_body)
        }
        .min(translated_body);
        let boundary = (speaker_prefix_len + projected)
            .max(new_boundary)
            .min(translated_chars.len());
        result.push((
            fragment.offset,
            translated_chars[new_boundary..boundary].iter().collect(),
        ));
        new_boundary = boundary;
    }
    let _ = entry;
    Ok(result)
}

fn find_dynamic_regions(program: &BytecodeProgram) -> BTreeSet<usize> {
    let mut skipped = BTreeSet::new();
    for function in &program.functions {
        let mut start = function.first_instruction;
        for index in function.first_instruction..function.instruction_end {
            if program.instructions[index].opcode == 2113 {
                mark_dynamic_region(program, start, index, &mut skipped);
                start = index + 1;
            }
        }
        mark_dynamic_region(program, start, function.instruction_end, &mut skipped);
    }
    skipped
}

fn mark_dynamic_region(
    program: &BytecodeProgram,
    start: usize,
    end: usize,
    skipped: &mut BTreeSet<usize>,
) {
    let region = &program.instructions[start..end];
    if !region.iter().any(|instruction| {
        instruction.opcode == 2114
            && instruction
                .text_bytes
                .as_deref()
                .is_some_and(|bytes| bytes.first() == Some(&b'd'))
    }) {
        return;
    }
    for instruction in region {
        if instruction.opcode == 2112 {
            skipped.insert(instruction.offset);
        }
    }
}

fn control_instructions_to_remove(program: &BytecodeProgram) -> BTreeSet<usize> {
    let mut remove = BTreeSet::new();
    for (index, instruction) in program.instructions.iter().enumerate() {
        if is_linebreak(instruction) {
            remove.insert(instruction.offset);
            if index > 0 && program.instructions[index - 1].opcode == 2113 {
                remove.insert(program.instructions[index - 1].offset);
            }
        }
        if instruction.opcode == 2049 {
            let ruby_pair = program.instructions.get(index + 1).is_some_and(|helper| {
                helper.opcode == 2064
                    && helper.script_call_hash == Some(RUBY_HELPER_HASH)
                    && helper.script_call_argument_count == Some(1)
                    && program
                        .instructions
                        .get(index + 2)
                        .is_some_and(|next| next.opcode == 2112)
            });
            if ruby_pair {
                remove.insert(instruction.offset);
                remove.insert(program.instructions[index + 1].offset);
                if index > 0 && program.instructions[index - 1].opcode == 2113 {
                    remove.insert(program.instructions[index - 1].offset);
                }
            }
        }
    }
    remove
}

fn is_linebreak(instruction: &Instruction) -> bool {
    instruction.opcode == 2114 && instruction.text_bytes.as_deref() == Some(b"n\0")
}

fn decode_operand(instruction: &Instruction) -> Result<TextFragment, String> {
    let bytes = instruction.text_bytes.as_deref().ok_or_else(|| {
        format!(
            "opcode {} at 0x{:X} has no parsed string",
            instruction.opcode, instruction.offset
        )
    })?;
    if bytes.is_empty() || bytes.last() != Some(&0) || bytes[..bytes.len() - 1].contains(&0) {
        return Err(format!(
            "opcode {} at 0x{:X} has an invalid NUL-terminated operand",
            instruction.opcode, instruction.offset
        ));
    }
    let (decoded, had_errors) =
        encoding_rs::SHIFT_JIS.decode_without_bom_handling(&bytes[..bytes.len() - 1]);
    if had_errors {
        return Err(format!(
            "opcode {} at 0x{:X} is invalid CP932",
            instruction.opcode, instruction.offset
        ));
    }
    let source_text = decoded.into_owned();
    let (display_text, physical_newline_positions) =
        strip_newlines_recording_positions(&source_text);
    Ok(TextFragment {
        offset: instruction.offset,
        opcode: instruction.opcode,
        source_text,
        display_text,
        source_bytes_hex: bytes_to_hex(bytes),
        physical_newline_positions,
    })
}

fn strip_newlines_recording_positions(text: &str) -> (String, Vec<usize>) {
    let mut output = String::with_capacity(text.len());
    let mut positions = Vec::new();
    for (scalar_index, character) in text.chars().enumerate() {
        if matches!(character, '\r' | '\n') {
            positions.push(scalar_index);
        } else {
            output.push(character);
        }
    }
    (output, positions)
}

fn decode_cp932_z(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() || bytes.last() != Some(&0) || bytes[..bytes.len() - 1].contains(&0) {
        return None;
    }
    let (text, had_errors) =
        encoding_rs::SHIFT_JIS.decode_without_bom_handling(&bytes[..bytes.len() - 1]);
    (!had_errors).then(|| text.into_owned())
}

fn strip_newlines(text: &str) -> String {
    text.chars()
        .filter(|character| !matches!(character, '\r' | '\n'))
        .collect()
}

fn strip_newline_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut output = bytes
        .iter()
        .copied()
        .filter(|byte| !matches!(*byte, b'\r' | b'\n'))
        .collect::<Vec<_>>();
    if output.last() != Some(&0) {
        output.push(0);
    }
    output
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02X}");
    }
    hex
}

fn refresh_translation_fields(record: &mut ScriptTextRecord) {
    let mut message = String::new();
    let mut event_index = 0usize;
    for fragment_index in 0..=record.fragments.len() {
        while record
            .events
            .get(event_index)
            .is_some_and(|event| event.before_fragment_index == fragment_index)
        {
            if let Some(token) = &record.events[event_index].visible_token {
                message.push_str(token);
            }
            event_index += 1;
        }
        if let Some(fragment) = record.fragments.get(fragment_index) {
            message.push_str(&fragment.display_text);
        }
    }
    if record.kind == RecordKind::ChoiceOption {
        if record.choice_quote_wrapped {
            message = message
                .strip_prefix('「')
                .and_then(|text| text.strip_suffix('」'))
                .unwrap_or(&message)
                .to_owned();
        }
        record.message = message;
        return;
    }
    match split_name_message(&message) {
        Some((name, body, closed)) => {
            record.name = Some(name.to_owned());
            record.message = body;
            record.speaker_quote_closed = closed;
        }
        None => {
            record.name = None;
            record.message = message;
            record.speaker_quote_closed = false;
        }
    }
}

fn split_name_message(text: &str) -> Option<(&str, String, bool)> {
    let (name, remainder) = text.split_once('「')?;
    if name.is_empty() || name.contains('「') || name.contains('」') {
        return None;
    }
    if let Some(message) = remainder.strip_suffix('」') {
        Some((name, format!("「{message}」"), true))
    } else {
        Some((name, format!("「{remainder}"), false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_opcode(payload: &mut Vec<u8>, opcode: u16) {
        payload.extend_from_slice(&opcode.to_le_bytes());
    }

    fn push_text(payload: &mut Vec<u8>, opcode: u16, text: &str) -> usize {
        let (encoded, _, had_errors) = encoding_rs::SHIFT_JIS.encode(text);
        assert!(!had_errors);
        let mut bytes = encoded.into_owned();
        bytes.push(0);
        let offset = payload.len();
        push_opcode(payload, opcode);
        payload.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        payload.extend_from_slice(&bytes);
        offset
    }

    fn push_call(payload: &mut Vec<u8>, opcode: u16, hash: u32, argc: u16) {
        push_opcode(payload, opcode);
        payload.extend_from_slice(&hash.to_le_bytes());
        payload.extend_from_slice(&0u32.to_le_bytes());
        payload.extend_from_slice(&argc.to_le_bytes());
    }

    #[test]
    fn removes_newline_and_ruby_controls_but_keeps_choices_and_relocations() {
        let mut payload = Vec::new();
        push_text(&mut payload, 2112, "A");
        let linebreak_display = payload.len();
        push_opcode(&mut payload, 2113);
        let newline = push_text(&mut payload, 2114, "n");
        push_opcode(&mut payload, 2106);
        payload.extend_from_slice(&0u16.to_le_bytes());
        push_text(&mut payload, 2112, "B");
        let ruby_display = payload.len();
        push_opcode(&mut payload, 2113);
        let ruby_reading = push_text(&mut payload, 2049, "にら");
        let ruby_call = payload.len();
        push_call(&mut payload, 2064, RUBY_HELPER_HASH, 1);
        let ruby_target = push_text(&mut payload, 2112, "睨みつける");
        push_opcode(&mut payload, 2113);
        let option_a = push_text(&mut payload, 2049, "「ひとり？」");
        let option_b = push_text(&mut payload, 2049, "「ふたり？」");
        let choice_call = payload.len();
        push_call(&mut payload, 2063, MENU_HELPER_HASH, 2);
        push_opcode(&mut payload, 2096);
        payload.extend_from_slice(&0i32.to_le_bytes());
        push_opcode(&mut payload, 2091);

        let program = crate::mjo_bytecode::parse_payload(&payload, &[(1, 0)], 0).unwrap();
        let report = extract_records_with_report(&payload, &program).unwrap();
        let display = report
            .records
            .iter()
            .find(|record| record.message == "AB睨みつける")
            .unwrap();
        assert_eq!(display.fragments.len(), 3);
        assert!(display
            .events
            .iter()
            .any(|event| event.kind == NativeEventKind::LineBreak));
        let ruby = display
            .events
            .iter()
            .find(|event| event.kind == NativeEventKind::Ruby)
            .unwrap();
        assert_eq!(ruby.source_text.as_deref(), Some("にら"));
        assert_eq!(ruby.related_offset, Some(ruby_target));
        assert!(report.removed_instruction_offsets.contains(&newline));
        assert!(report
            .removed_instruction_offsets
            .contains(&linebreak_display));
        assert!(report.removed_instruction_offsets.contains(&ruby_call));
        assert!(report.removed_instruction_offsets.contains(&ruby_reading));
        assert!(report.removed_instruction_offsets.contains(&ruby_display));

        let choices = report
            .records
            .iter()
            .filter(|record| record.kind == RecordKind::ChoiceOption)
            .collect::<Vec<_>>();
        assert_eq!(
            choices
                .iter()
                .map(|record| record.message.as_str())
                .collect::<Vec<_>>(),
            ["ふたり？", "ひとり？"]
        );
        assert!(choices.iter().all(|record| {
            record.choice_control.as_ref().is_some_and(|control| {
                control.call_offset == choice_call
                    && control.flow_edges.iter().any(|edge| edge.opcode == 2096)
            })
        }));
        assert_ne!(option_a, option_b);

        let entries = translation_entries(&report.records);
        let edits = build_edits(&payload, &program, &report, &entries, |text| {
            let (encoded, _, errors) = encoding_rs::SHIFT_JIS.encode(text);
            if errors {
                Err("not CP932".into())
            } else {
                Ok(encoded.into_owned())
            }
        })
        .unwrap();
        let rebuilt = crate::mjo_rebuild_clean::rebuild_payload(
            &payload,
            &program,
            &edits.replacement_operands,
            &edits.removed_instruction_offsets,
            &[(1, 0)],
            0,
        )
        .unwrap();
        let rebuilt_program = crate::mjo_bytecode::parse_payload(
            &rebuilt.payload,
            &rebuilt.symbols,
            rebuilt.entrypoint,
        )
        .unwrap();
        assert!(!rebuilt_program.instructions.iter().any(is_linebreak));
        assert!(!rebuilt_program
            .instructions
            .iter()
            .any(|instruction| instruction.script_call_hash == Some(RUBY_HELPER_HASH)));
        let rebuilt_report =
            extract_records_with_report(&rebuilt.payload, &rebuilt_program).unwrap();
        assert!(rebuilt_report
            .records
            .iter()
            .any(|record| record.message == "AB睨みつける"));
        assert_eq!(rebuilt_report.choice_option_count, 2);
    }
}
