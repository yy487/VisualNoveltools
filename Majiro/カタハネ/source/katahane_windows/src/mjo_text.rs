//! Translator-facing projection for Majiro opcode 2112 display strings.

use std::collections::{BTreeMap, BTreeSet};

use super::mjo_bytecode::{reachable_instruction_indices, BytecodeProgram};

/// `yazlib.mjo`'s confirmed ruby-reading helper.
const RUBY_HELPER_HASH: u32 = 0x3198_FD01;
/// The menu script helper receives the option labels as its complete argument list.
const CHOICE_HELPER_HASH: u32 = 0x0A7A_489C;

const LINE_BREAK_TOKEN: &str = "\n";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TextFragment {
    pub(super) offset: usize,
    pub(super) opcode: u16,
    /// Decoded source fragment, retaining every literal CR/LF byte.
    pub(super) source_text: String,
    /// Original length-prefixed operand, including its terminal NUL, as hex.
    pub(super) source_bytes_hex: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecordKind {
    Display,
    ChoiceOption,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MarkerKind {
    Voice,
    Dynamic,
    LineBreak,
    Ruby,
    NativeOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProtectedMarker {
    /// Index of the first text fragment following this unchanged instruction.
    pub(super) before_fragment_index: usize,
    /// Payload-relative offset of the unchanged instruction.
    pub(super) offset: usize,
    pub(super) opcode: u16,
    pub(super) kind: MarkerKind,
    pub(super) source_text: Option<String>,
    /// Original length-prefixed operand, including NUL, as hex.
    pub(super) source_bytes_hex: Option<String>,
    /// The 2112 operand associated with the ruby helper, when confirmed.
    pub(super) ruby_target_offset: Option<usize>,
    /// Translator-facing protected token; structural-only events keep this empty.
    pub(super) token: String,
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
    pub(super) option_index: u16,
    pub(super) option_offsets: Vec<usize>,
    pub(super) flow_edges: Vec<ChoiceFlowEdge>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ScriptTextRecord {
    pub(super) kind: RecordKind,
    /// First payload-relative text instruction offset.
    pub(super) offset: usize,
    /// Source opcode, always 2112.
    pub(super) opcode: u16,
    /// Speaker prefix from a source segment beginning with `name「`.
    pub(super) name: Option<String>,
    /// Whether the source text ends with the speaker form's closing `」`.
    pub(super) speaker_quote_closed: bool,
    /// Menu labels are displayed without their source `「...」` wrapper.
    pub(super) choice_quote_wrapped: bool,
    /// The shared menu function call and the post-call branch edges.
    pub(super) choice_control: Option<ChoiceControl>,
    /// One or more source instructions that form the translator-facing record.
    pub(super) fragments: Vec<TextFragment>,
    /// Non-2112 string/dynamic instructions retained at their exact positions.
    pub(super) markers: Vec<ProtectedMarker>,
    /// Translator-facing message body, or the full text when it has no speaker prefix.
    pub(super) message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ExtractionReport {
    /// Confirmed opcode 2112 display text.
    pub(super) records: Vec<ScriptTextRecord>,
    /// Number of exported opcode 2112 direct-display records.
    pub(super) direct_2112_record_count: usize,
    /// Independently editable static menu labels passed to the menu helper.
    pub(super) choice_option_count: usize,
    /// Distinct calls into the confirmed menu helper.
    pub(super) choice_group_count: usize,
    /// Number of opcode 2114 dynamic-format operands encountered.
    pub(super) dynamic_instruction_count: usize,
    /// Number of opcode 2114 operands whose operation string starts with `d`.
    pub(super) dynamic_append_instruction_count: usize,
    /// Number of protected instruction placeholders present in exported records.
    pub(super) protected_marker_count: usize,
    /// Opcode 2112 offsets withheld because their local 2113-delimited region
    /// also contains an opcode 2114 `d` operation.
    pub(super) skipped_dynamic_offsets: Vec<usize>,
    /// Opcode 2112 offsets with no control-flow path from any symbol entry.
    pub(super) skipped_unreachable_offsets: Vec<usize>,
    /// Whitespace-only 2112 operands kept unchanged and omitted from JSON.
    pub(super) skipped_blank_offsets: Vec<usize>,
}

/// Extract records and retain a report for CLI warnings about dynamic text.
///
/// Opcode 2113 is not treated as a sentence boundary. Adjacent 2112 fragments
/// may be merged when an open speaker quote continues across display/control
/// instructions, or when a mid-sentence line break splits them. Interstitial
/// 2049 strings and 2114 `d` operations are represented by protected markers;
/// their instructions remain unchanged during import.
pub(super) fn extract_records_with_report(
    payload: &[u8],
    program: &BytecodeProgram,
) -> Result<ExtractionReport, String> {
    let mut report = extract_records_with_report_mode(payload, program)?;
    assign_readable_marker_tokens(&mut report.records);
    Ok(report)
}

fn extract_records_with_report_mode(
    payload: &[u8],
    program: &BytecodeProgram,
) -> Result<ExtractionReport, String> {
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
    let reachable = reachable_instruction_indices(program);
    let mut skipped = vec![false; program.instructions.len()];

    for function in &program.functions {
        let mut segment_start = function.first_instruction;
        for index in function.first_instruction..function.instruction_end {
            if program.instructions[index].opcode == 2113 {
                mark_dynamic_region(program, segment_start, index, &mut skipped);
                segment_start = index + 1;
            }
        }
        mark_dynamic_region(
            program,
            segment_start,
            function.instruction_end,
            &mut skipped,
        );
    }

    for (instruction_index, instruction) in program.instructions.iter().enumerate() {
        if instruction.opcode != 2112 {
            continue;
        }
        if !reachable.contains(&instruction_index) {
            report.skipped_unreachable_offsets.push(instruction.offset);
            continue;
        }
        let bytes = instruction.text_bytes.as_deref().ok_or_else(|| {
            format!(
                "opcode {} at payload offset 0x{:X} has no parsed string operand",
                instruction.opcode, instruction.offset
            )
        })?;
        let expected_size = bytes.len().checked_add(4).ok_or_else(|| {
            format!(
                "opcode {} at 0x{:X} size overflow",
                instruction.opcode, instruction.offset
            )
        })?;
        let instruction_end = instruction
            .offset
            .checked_add(instruction.size)
            .ok_or_else(|| {
                format!(
                    "opcode {} at 0x{:X} range overflow",
                    instruction.opcode, instruction.offset
                )
            })?;
        let source = payload
            .get(instruction.offset..instruction_end)
            .ok_or_else(|| {
                format!(
                    "opcode {} at 0x{:X} exceeds payload",
                    instruction.opcode, instruction.offset
                )
            })?;
        let source_length = source.get(2..4).ok_or_else(|| {
            format!(
                "opcode {} at 0x{:X} has a truncated length",
                instruction.opcode, instruction.offset
            )
        })?;
        let declared_length = u16::from_le_bytes([source_length[0], source_length[1]]) as usize;
        let opcode_bytes = instruction.opcode.to_le_bytes();
        if instruction.size != expected_size
            || source.get(..2) != Some(&opcode_bytes[..])
            || declared_length != bytes.len()
            || source.get(4..) != Some(bytes)
        {
            return Err(format!(
                "opcode {} metadata at payload offset 0x{:X} does not match the supplied payload",
                instruction.opcode, instruction.offset
            ));
        }
        if bytes.is_empty() || bytes.last() != Some(&0) {
            return Err(format!(
                "opcode {} at payload offset 0x{:X} must have exactly one terminal NUL",
                instruction.opcode, instruction.offset
            ));
        }
        let content = &bytes[..bytes.len() - 1];
        if content.contains(&0) {
            return Err(format!(
                "opcode {} at payload offset 0x{:X} contains an internal NUL",
                instruction.opcode, instruction.offset
            ));
        }

        let (decoded, had_errors) = encoding_rs::SHIFT_JIS.decode_without_bom_handling(content);
        if had_errors {
            return Err(format!(
                "opcode {} at payload offset 0x{:X} is not valid Shift-JIS/CP932",
                instruction.opcode, instruction.offset
            ));
        }
        let decoded = decoded.into_owned();
        if decoded.chars().all(char::is_whitespace) {
            report.skipped_blank_offsets.push(instruction.offset);
            continue;
        }
        let source_text = decoded.clone();
        let (name, message, speaker_quote_closed) = match split_name_message(&decoded) {
            Some((name, message, closed)) => (Some(name.to_owned()), message, closed),
            None => (None, decoded, false),
        };

        let record = ScriptTextRecord {
            offset: instruction.offset,
            opcode: instruction.opcode,
            name,
            speaker_quote_closed,
            kind: RecordKind::Display,
            choice_quote_wrapped: false,
            choice_control: None,
            fragments: vec![TextFragment {
                offset: instruction.offset,
                opcode: instruction.opcode,
                source_text,
                source_bytes_hex: bytes_to_hex(bytes),
            }],
            markers: Vec::new(),
            message,
        };
        report.records.push(record);
        report.direct_2112_record_count += 1;
    }

    report.records = merge_wrapped_dialogue_fragments(report.records, program);
    let (choice_records, choice_group_count) =
        extract_choice_records(payload, program, &reachable)?;
    report.choice_option_count = choice_records.len();
    report.choice_group_count = choice_group_count;
    report.records.extend(choice_records);

    // A record that carries a dynamic insertion token is complete for
    // translation, so unhide all its 2112 fragments from the conservative
    // dynamic-region filter.
    let represented_dynamic_fragments = report
        .records
        .iter()
        .filter(|record| {
            record
                .markers
                .iter()
                .any(|marker| marker.kind == MarkerKind::Dynamic)
        })
        .flat_map(|record| record.fragments.iter().map(|fragment| fragment.offset))
        .collect::<std::collections::BTreeSet<_>>();
    for (index, instruction) in program.instructions.iter().enumerate() {
        if instruction.opcode == 2112
            && reachable.contains(&index)
            && represented_dynamic_fragments.contains(&instruction.offset)
        {
            skipped[index] = false;
        }
        if instruction.opcode == 2112 && reachable.contains(&index) && skipped[index] {
            report.skipped_dynamic_offsets.push(instruction.offset);
        }
    }
    let skipped_offsets = report
        .skipped_dynamic_offsets
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    report.records.retain(|record| {
        !record
            .fragments
            .iter()
            .any(|fragment| skipped_offsets.contains(&fragment.offset))
    });
    report.protected_marker_count = report
        .records
        .iter()
        .map(|record| record.markers.len())
        .sum();
    report
        .records
        .sort_by_key(|record| match &record.choice_control {
            Some(control) => (control.call_offset, 1u8, control.option_index),
            None => (record.offset, 0u8, 0u16),
        });

    Ok(report)
}

fn assign_readable_marker_tokens(records: &mut [ScriptTextRecord]) {
    for record in records {
        let source_text = record
            .name
            .iter()
            .chain(
                record
                    .fragments
                    .iter()
                    .map(|fragment| &fragment.source_text),
            )
            .fold(String::new(), |mut text, part| {
                text.push_str(part);
                text
            });
        let voice_token =
            readable_marker_token(&source_text, "\\v", &["〔语音参数〕", "〔语音参数备用〕"]);
        let dynamic_token =
            readable_marker_token(&source_text, "\\d", &["〔动态值〕", "〔动态备用〕"]);
        for marker in &mut record.markers {
            match marker.kind {
                MarkerKind::Voice => marker.token.clone_from(&voice_token),
                MarkerKind::Dynamic => marker.token.clone_from(&dynamic_token),
                MarkerKind::LineBreak => marker.token = LINE_BREAK_TOKEN.to_owned(),
                MarkerKind::Ruby | MarkerKind::NativeOnly => marker.token.clear(),
            }
        }
        refresh_translator_fields(record);
    }
}

fn readable_marker_token(source: &str, preferred: &str, fallbacks: &[&str]) -> String {
    if !source.contains(preferred) {
        return preferred.to_owned();
    }
    fallbacks
        .iter()
        .find(|candidate| !source.contains(**candidate))
        .copied()
        .unwrap_or(preferred)
        .to_owned()
}

fn merge_wrapped_dialogue_fragments(
    records: Vec<ScriptTextRecord>,
    program: &BytecodeProgram,
) -> Vec<ScriptTextRecord> {
    let mut merged = Vec::with_capacity(records.len());
    let mut index = 0usize;
    while index < records.len() {
        let mut current = records[index].clone();
        let mut consumed = 1usize;
        loop {
            let Some(next) = records.get(index + consumed) else {
                break;
            };
            if next.opcode != 2112 || next.name.is_some() {
                break;
            }
            let Some(previous_fragment) = current.fragments.last() else {
                break;
            };
            let Some(mut bridge_markers) =
                protected_markers_between(program, previous_fragment.offset, next.offset)
            else {
                break;
            };

            let previous_text = &previous_fragment.source_text;
            let next_text = &next.fragments[0].source_text;
            let open_speaker = current.name.is_some() && !current.speaker_quote_closed;
            let linebreak_boundary = has_linebreak_boundary(previous_text, next_text);
            let structured_boundary = bridge_markers.iter().any(|marker| {
                matches!(
                    marker.kind,
                    MarkerKind::LineBreak
                        | MarkerKind::Ruby
                        | MarkerKind::Voice
                        | MarkerKind::Dynamic
                )
            });
            if !open_speaker && !linebreak_boundary && !structured_boundary {
                if !bridge_markers.is_empty() {
                    let boundary = current.fragments.len();
                    for mut marker in bridge_markers {
                        marker.before_fragment_index = boundary;
                        current.markers.push(marker);
                    }
                    refresh_translator_fields(&mut current);
                }
                break;
            }

            let next_fragment_index = current.fragments.len();
            for marker in &mut bridge_markers {
                marker.before_fragment_index = next_fragment_index;
            }
            current.markers.extend(bridge_markers);
            current.fragments.push(next.fragments[0].clone());
            refresh_translator_fields(&mut current);
            consumed += 1;

            if current.speaker_quote_closed {
                break;
            }
        }
        merged.push(current);
        index += consumed;
    }
    merged
}

fn refresh_translator_fields(record: &mut ScriptTextRecord) {
    let text = composed_translation_text(record);
    if record.kind == RecordKind::ChoiceOption {
        record.name = None;
        record.message = if record.choice_quote_wrapped {
            text.strip_prefix('「')
                .and_then(|text| text.strip_suffix('」'))
                .unwrap_or(&text)
                .to_owned()
        } else {
            text
        };
        record.speaker_quote_closed = false;
        return;
    }
    match split_name_message(&text) {
        Some((name, message, closed)) => {
            record.name = Some(name.to_owned());
            record.message = message;
            record.speaker_quote_closed = closed;
        }
        None => {
            record.name = None;
            record.message = text;
            record.speaker_quote_closed = false;
        }
    }
}

fn composed_source_text(record: &ScriptTextRecord) -> String {
    let mut text = String::new();
    let mut marker_index = 0usize;
    for fragment_index in 0..=record.fragments.len() {
        while record
            .markers
            .get(marker_index)
            .is_some_and(|marker| marker.before_fragment_index == fragment_index)
        {
            text.push_str(&record.markers[marker_index].token);
            marker_index += 1;
        }
        if let Some(fragment) = record.fragments.get(fragment_index) {
            text.push_str(&fragment.source_text);
        }
    }
    text
}

fn composed_translation_text(record: &ScriptTextRecord) -> String {
    let mut text = String::new();
    let mut marker_index = 0usize;
    for fragment_index in 0..=record.fragments.len() {
        while record
            .markers
            .get(marker_index)
            .is_some_and(|marker| marker.before_fragment_index == fragment_index)
        {
            let marker = &record.markers[marker_index];
            if marker.kind != MarkerKind::LineBreak {
                text.push_str(&marker.token);
            }
            marker_index += 1;
        }
        if let Some(fragment) = record.fragments.get(fragment_index) {
            text.push_str(&fragment.source_text);
        }
    }
    text
}

pub(super) fn primary_source_text(record: &ScriptTextRecord) -> String {
    composed_source_text(record)
}

fn protected_markers_between(
    program: &BytecodeProgram,
    first_offset: usize,
    second_offset: usize,
) -> Option<Vec<ProtectedMarker>> {
    let Ok(first) = program
        .instructions
        .binary_search_by_key(&first_offset, |instruction| instruction.offset)
    else {
        return None;
    };
    let Ok(second) = program
        .instructions
        .binary_search_by_key(&second_offset, |instruction| instruction.offset)
    else {
        return None;
    };
    let function_start = |offset| {
        program
            .functions
            .iter()
            .find(|function| offset >= function.start && offset < function.end)
            .map(|function| function.start)
    };
    if first >= second || function_start(first_offset) != function_start(second_offset) {
        return None;
    }
    let mut markers = Vec::new();
    for (bridge_index, instruction) in program.instructions[first + 1..second].iter().enumerate() {
        match instruction.opcode {
            2049 => {
                let bytes = instruction.text_bytes.as_deref()?;
                if bytes.is_empty()
                    || bytes.last() != Some(&0)
                    || bytes[..bytes.len() - 1].contains(&0)
                {
                    return None;
                }
                if bytes.len() > 1 {
                    let key = decode_cp932_z(bytes)?;
                    let next = program.instructions.get(first + 1 + bridge_index + 2);
                    let helper = program.instructions.get(first + 1 + bridge_index + 1);
                    let ruby = helper.is_some_and(|helper| {
                        helper.opcode == 2064
                            && helper.script_call_hash == Some(RUBY_HELPER_HASH)
                            && next.is_some_and(|next| next.opcode == 2112)
                    });
                    let (ruby_target_offset, ruby_start_char, ruby_length_chars) = if ruby {
                        let target = next.expect("ruby helper target is opcode 2112");
                        let span = target
                            .text_bytes
                            .as_deref()
                            .and_then(decode_cp932_z)
                            .and_then(|text| ruby_base_span(&text));
                        (
                            Some(target.offset),
                            span.map(|(start, _)| start),
                            span.map(|(_, length)| length),
                        )
                    } else {
                        (None, None, None)
                    };
                    let kind = if is_voice_identifier(&key) {
                        MarkerKind::Voice
                    } else if ruby {
                        MarkerKind::Ruby
                    } else {
                        MarkerKind::Annotation
                    };
                    let token = match kind {
                        MarkerKind::Voice | MarkerKind::Dynamic => String::new(),
                        MarkerKind::Ruby => format!("〔注音:{key}〕"),
                        MarkerKind::Annotation => format!("〔注记:{key}〕"),
                        MarkerKind::LineBreak => LINE_BREAK_TOKEN.to_owned(),
                    };
                    markers.push(ProtectedMarker {
                        before_fragment_index: 0,
                        offset: instruction.offset,
                        opcode: 2049,
                        kind,
                        source_text: Some(key),
                        source_bytes_hex: Some(bytes_to_hex(bytes)),
                        ruby_target_offset,
                        ruby_start_char,
                        ruby_length_chars,
                        token,
                    });
                }
            }
            2114 => {
                let bytes = instruction.text_bytes.as_deref()?;
                if bytes.first() == Some(&b'd') {
                    markers.push(ProtectedMarker {
                        before_fragment_index: 0,
                        offset: instruction.offset,
                        opcode: 2114,
                        kind: MarkerKind::Dynamic,
                        source_text: decode_cp932_z(bytes),
                        source_bytes_hex: Some(bytes_to_hex(bytes)),
                        ruby_target_offset: None,
                        ruby_start_char: None,
                        ruby_length_chars: None,
                        token: String::new(),
                    });
                } else if bytes == b"n\0" {
                    markers.push(ProtectedMarker {
                        before_fragment_index: 0,
                        offset: instruction.offset,
                        opcode: 2114,
                        kind: MarkerKind::LineBreak,
                        source_text: Some("n".into()),
                        source_bytes_hex: Some(bytes_to_hex(bytes)),
                        ruby_target_offset: None,
                        ruby_start_char: None,
                        ruby_length_chars: None,
                        token: LINE_BREAK_TOKEN.to_owned(),
                    });
                }
            }
            2064 if instruction.script_call_hash == Some(RUBY_HELPER_HASH) => {
                let previous_index = (first + 1 + bridge_index).checked_sub(1)?;
                let previous = program.instructions.get(previous_index)?;
                let next = program.instructions.get(first + 1 + bridge_index + 1)?;
                let voice_key = previous.text_bytes.as_deref()?;
                if previous.opcode != 2049
                    || voice_key.len() <= 1
                    || voice_key.last() != Some(&0)
                    || voice_key[..voice_key.len() - 1].contains(&0)
                    || next.opcode != 2112
                {
                    return None;
                }
            }
            2063
            | 2091
            | 2095
            | 2100
            | 2101
            | 2112
            | 2115
            | 2117
            | 2119
            | 2128
            | 2092..=2105
            | 2107..=2109 => return None,
            _ => {}
        }
    }
    Some(markers)
}

fn decode_cp932_z(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() || bytes.last() != Some(&0) || bytes[..bytes.len() - 1].contains(&0) {
        return None;
    }
    let (decoded, had_errors) =
        encoding_rs::SHIFT_JIS.decode_without_bom_handling(&bytes[..bytes.len() - 1]);
    (!had_errors).then(|| decoded.into_owned())
}

fn extract_choice_records(
    payload: &[u8],
    program: &BytecodeProgram,
    reachable: &std::collections::BTreeSet<usize>,
) -> Result<(Vec<ScriptTextRecord>, usize), String> {
    let mut records = Vec::new();
    let mut groups = 0usize;
    for (call_index, call) in program.instructions.iter().enumerate() {
        if call.opcode != 2063 || call.script_call_hash != Some(CHOICE_HELPER_HASH) {
            continue;
        }
        if !reachable.contains(&call_index) {
            continue;
        }
        let argument_count = call.script_call_argument_count.ok_or_else(|| {
            format!(
                "menu helper at 0x{:X} has no parsed argument count",
                call.offset
            )
        })?;
        if argument_count < 2 {
            return Err(format!(
                "menu helper at 0x{:X} has implausible option count {argument_count}",
                call.offset
            ));
        }
        let first_option_index = call_index
            .checked_sub(usize::from(argument_count))
            .ok_or_else(|| format!("menu helper at 0x{:X} lacks option operands", call.offset))?;
        let function = program
            .functions
            .iter()
            .find(|function| {
                call_index >= function.first_instruction && call_index < function.instruction_end
            })
            .ok_or_else(|| {
                format!(
                    "menu helper at 0x{:X} is outside a parsed function",
                    call.offset
                )
            })?;
        if first_option_index < function.first_instruction {
            return Err(format!(
                "menu helper at 0x{:X} crosses a function boundary",
                call.offset
            ));
        }
        let option_instructions = &program.instructions[first_option_index..call_index];
        if option_instructions.len() != usize::from(argument_count)
            || option_instructions
                .iter()
                .any(|instruction| instruction.opcode != 2049)
        {
            return Err(format!(
                "menu helper at 0x{:X} argc={argument_count} is not preceded by that many static 2049 option strings",
                call.offset
            ));
        }
        if option_instructions.iter().any(|instruction| {
            program
                .instructions
                .binary_search_by_key(&instruction.offset, |candidate| candidate.offset)
                .ok()
                .is_none_or(|index| !reachable.contains(&index))
        }) {
            return Err(format!(
                "menu helper at 0x{:X} has an unreachable option operand",
                call.offset
            ));
        }
        groups += 1;
        let option_offsets = option_instructions
            .iter()
            .map(|instruction| instruction.offset)
            .collect::<Vec<_>>();
        let flow_edges = choice_flow_edges(program, function.instruction_end, call_index);

        // The VM helper consumes the argument stack from the top, so the last
        // 2049 pushed is the first item presented in the menu.
        for (option_index, instruction) in option_instructions.iter().rev().enumerate() {
            let bytes = instruction.text_bytes.as_deref().ok_or_else(|| {
                format!(
                    "choice operand 2049 at 0x{:X} has no parsed bytes",
                    instruction.offset
                )
            })?;
            let end = instruction
                .offset
                .checked_add(instruction.size)
                .ok_or_else(|| {
                    format!(
                        "choice operand at 0x{:X} range overflows",
                        instruction.offset
                    )
                })?;
            let raw = payload.get(instruction.offset..end).ok_or_else(|| {
                format!(
                    "choice operand at 0x{:X} exceeds the MJO payload",
                    instruction.offset
                )
            })?;
            let length = raw.get(2..4).ok_or_else(|| {
                format!(
                    "choice operand at 0x{:X} has a truncated length",
                    instruction.offset
                )
            })?;
            let declared_length = u16::from_le_bytes([length[0], length[1]]) as usize;
            if raw.get(..2) != Some(&2049u16.to_le_bytes()[..])
                || declared_length != bytes.len()
                || raw.get(4..) != Some(bytes)
            {
                return Err(format!(
                    "choice operand metadata at 0x{:X} does not match its source bytes",
                    instruction.offset
                ));
            }
            let source_text = decode_cp932_z(bytes).ok_or_else(|| {
                format!(
                    "choice operand 2049 at 0x{:X} is not a valid CP932 string",
                    instruction.offset
                )
            })?;
            if source_text.chars().all(char::is_whitespace) {
                return Err(format!(
                    "choice operand 2049 at 0x{:X} is blank",
                    instruction.offset
                ));
            }
            let choice_quote_wrapped = source_text.starts_with('「') && source_text.ends_with('」');
            let message = if choice_quote_wrapped {
                source_text
                    .strip_prefix('「')
                    .and_then(|text| text.strip_suffix('」'))
                    .unwrap_or(&source_text)
                    .to_owned()
            } else {
                source_text.clone()
            };
            records.push(ScriptTextRecord {
                kind: RecordKind::ChoiceOption,
                offset: instruction.offset,
                opcode: instruction.opcode,
                name: None,
                speaker_quote_closed: false,
                choice_quote_wrapped,
                choice_control: Some(ChoiceControl {
                    call_offset: call.offset,
                    callee_hash: CHOICE_HELPER_HASH,
                    argument_count,
                    option_index: u16::try_from(option_index)
                        .map_err(|_| "menu option index exceeds u16".to_string())?,
                    option_offsets: option_offsets.clone(),
                    flow_edges: flow_edges.clone(),
                }),
                fragments: vec![TextFragment {
                    offset: instruction.offset,
                    opcode: instruction.opcode,
                    source_text,
                    source_bytes_hex: bytes_to_hex(bytes),
                }],
                markers: Vec::new(),
                message,
            });
        }
    }
    Ok((records, groups))
}

fn choice_flow_edges(
    program: &BytecodeProgram,
    function_end: usize,
    call_index: usize,
) -> Vec<ChoiceFlowEdge> {
    let first = call_index + 1;
    let end = (first..function_end)
        .find(|index| matches!(program.instructions[*index].opcode, 2112 | 2091))
        .unwrap_or(function_end);
    let control_offsets = program.instructions[first..end]
        .iter()
        .map(|instruction| instruction.offset)
        .collect::<std::collections::BTreeSet<_>>();
    program
        .relocations
        .iter()
        .filter(|relocation| control_offsets.contains(&relocation.instruction_offset))
        .filter_map(|relocation| {
            let instruction_index = program
                .instructions
                .binary_search_by_key(&relocation.instruction_offset, |instruction| {
                    instruction.offset
                })
                .ok()?;
            Some(ChoiceFlowEdge {
                instruction_offset: relocation.instruction_offset,
                opcode: program.instructions[instruction_index].opcode,
                target_offset: relocation.target_offset,
            })
        })
        .collect()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02X}");
    }
    hex
}

fn is_voice_identifier(text: &str) -> bool {
    text.starts_with('v')
        && text.len() > 1
        && text
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

fn has_linebreak_boundary(left: &str, right: &str) -> bool {
    has_trailing_linebreak(left) || right.starts_with(['\r', '\n'])
}

fn has_trailing_linebreak(text: &str) -> bool {
    text.ends_with(['\r', '\n'])
}

fn ends_sentence_before_linebreak(text: &str) -> bool {
    let trimmed = text.trim_end_matches(['\r', '\n']).trim_end();
    trimmed.chars().last().is_some_and(|character| {
        matches!(
            character,
            '。' | '！' | '？' | '!' | '?' | '」' | '』' | '…'
        )
    })
}

fn ends_with_continuation(text: &str) -> bool {
    let text = text.trim_end();
    [
        "から",
        "まで",
        "より",
        "ので",
        "のに",
        "けれど",
        "けど",
        "ながら",
        "つつ",
        "たら",
        "なら",
        "ば",
        "って",
        "て",
        "で",
        "し",
    ]
    .iter()
    .any(|ending| text.ends_with(ending))
        || text.chars().last().is_some_and(|character| {
            matches!(
                character,
                'の' | 'が' | 'を' | 'に' | 'と' | 'で' | 'へ' | 'は' | 'も' | 'や' | '、'
            )
        })
}

fn splits_honorific_prefix(left: &str, right: &str) -> bool {
    let left = left.trim_end();
    let right = right.trim_start();
    let Some(prefix) = left.chars().last() else {
        return false;
    };
    let Some(next) = right.chars().next() else {
        return false;
    };
    matches!(prefix, 'お' | 'ご' | '御') && is_cjk_ideograph(next)
}

fn is_cjk_ideograph(character: char) -> bool {
    matches!(character as u32,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x20000..=0x2FA1F
    )
}

fn ruby_base_span(text: &str) -> Option<(usize, usize)> {
    let body_start_byte = text.find('「').map_or(0, |offset| offset + '「'.len_utf8());
    let body = text.get(body_start_byte..)?;
    let (relative_byte, _first) = body
        .char_indices()
        .find(|(_, character)| is_cjk_ideograph(*character))?;
    let mut length = 0usize;
    for character in body[relative_byte..].chars() {
        if !is_cjk_ideograph(character) {
            break;
        }
        length += 1;
    }
    let absolute_byte = body_start_byte + relative_byte;
    let start_char = text[..absolute_byte].chars().count();
    (length > 0).then_some((start_char, length))
}

fn mark_dynamic_region(program: &BytecodeProgram, start: usize, end: usize, skipped: &mut [bool]) {
    let instructions = &program.instructions[start..end];
    if !instructions.iter().any(|instruction| {
        instruction.opcode == 2114
            && instruction
                .text_bytes
                .as_deref()
                .is_some_and(|bytes| bytes.first() == Some(&b'd'))
    }) {
        return;
    }
    for (index, instruction) in program.instructions[start..end].iter().enumerate() {
        if instruction.opcode == 2112 {
            skipped[start + index] = true;
        }
    }
}

/// Project primary script records into vn-text's translator-facing entries.
pub(super) fn translation_entries(records: &[ScriptTextRecord]) -> Vec<vn_text::Entry> {
    records
        .iter()
        .map(|record| vn_text::Entry {
            name: record.name.clone(),
            message: record.message.clone(),
        })
        .collect()
}

/// Restore a translator-facing entry to the display string encoded in its
/// original opcode 2049 or 2112 operand.
pub(super) fn translated_text(
    record: &ScriptTextRecord,
    entry: &vn_text::Entry,
) -> Result<String, String> {
    if !matches!(
        (record.kind, record.opcode),
        (RecordKind::Display, 2112) | (RecordKind::ChoiceOption, 2049)
    ) {
        return Err(format!(
            "unsupported translation record opcode {}",
            record.opcode
        ));
    }
    if record.name.is_some() != entry.name.is_some() {
        return Err("translation changed whether this entry has a speaker name".into());
    }
    validate_protected_markers(record, &entry.message)?;
    if entry.name.is_some() {
        if !entry.message.starts_with('「') {
            return Err("message must retain the source speaker quote opener '「'".into());
        }
        if record.speaker_quote_closed && !entry.message.ends_with('」') {
            return Err("message must retain the source speaker quote closer '」'".into());
        }
    }
    if record.kind == RecordKind::ChoiceOption && entry.name.is_some() {
        return Err("a choice option cannot have a speaker name".into());
    }
    if record.kind == RecordKind::ChoiceOption && record.choice_quote_wrapped {
        return Ok(format!("「{}」", entry.message));
    }
    Ok(match &entry.name {
        Some(name) => format!("{name}{}", entry.message),
        None => entry.message.clone(),
    })
}

pub(super) fn validate_protected_markers(
    record: &ScriptTextRecord,
    message: &str,
) -> Result<(), String> {
    let mut expected = BTreeMap::<String, usize>::new();
    for marker in &record.markers {
        if marker.kind != MarkerKind::LineBreak && !marker.token.is_empty() {
            *expected.entry(marker.token.clone()).or_default() += 1;
        }
    }
    let mut positions = BTreeMap::<String, Vec<usize>>::new();
    for token in expected.keys() {
        positions.insert(
            token.clone(),
            message
                .match_indices(token)
                .map(|(position, _)| position)
                .collect(),
        );
    }

    let mut missing = BTreeSet::new();
    let mut repeated = BTreeSet::new();
    let mut reordered = BTreeSet::new();
    for (token, expected_count) in &expected {
        let actual_count = positions.get(token).map_or(0, Vec::len);
        if actual_count < *expected_count {
            missing.insert(token.as_str());
        } else if actual_count > *expected_count {
            repeated.insert(token.as_str());
        }
    }

    let mut previous_position = 0usize;
    if missing.is_empty() && repeated.is_empty() {
        let mut consumed = BTreeMap::<&str, usize>::new();
        for marker in &record.markers {
            let index = consumed.entry(marker.token.as_str()).or_default();
            let position = positions[&marker.token][*index];
            *index += 1;
            if position < previous_position {
                reordered.insert(marker.token.as_str());
            } else {
                previous_position = position + marker.token.len();
            }
        }
    }
    if missing.is_empty() && repeated.is_empty() && reordered.is_empty() {
        return Ok(());
    }
    let mut problems = Vec::new();
    if !missing.is_empty() {
        problems.push(format!(
            "缺少 [{}]",
            missing.iter().copied().collect::<Vec<_>>().join(", ")
        ));
    }
    if !repeated.is_empty() {
        problems.push(format!(
            "重复 [{}]",
            repeated.iter().copied().collect::<Vec<_>>().join(", ")
        ));
    }
    if !reordered.is_empty() {
        problems.push(format!(
            "顺序错误 [{}]",
            reordered.iter().copied().collect::<Vec<_>>().join(", ")
        ));
    }
    Err(format!(
        "受保护占位符必须按原数量、原顺序保留：{}",
        problems.join("；")
    ))
}

pub(super) fn distribute_translation(
    record: &ScriptTextRecord,
    entry: &vn_text::Entry,
    translated: &str,
) -> Result<Vec<(usize, String)>, String> {
    if record.markers.is_empty() {
        return distribute_to_fragment_range(record, entry, 0, record.fragments.len(), translated);
    }

    let mut parts = Vec::with_capacity(record.fragments.len());
    let mut fragment_start = 0usize;
    let mut marker_index = 0usize;
    let mut remaining = translated;
    while marker_index < record.markers.len() {
        let boundary = record.markers[marker_index].before_fragment_index;
        if boundary < fragment_start || boundary > record.fragments.len() {
            return Err("受保护占位符定位超出文本片段范围".into());
        }
        let mut group_end = marker_index + 1;
        while group_end < record.markers.len()
            && record.markers[group_end].before_fragment_index == boundary
        {
            group_end += 1;
        }
        let first_token = &record.markers[marker_index].token;
        let Some(token_position) = remaining.find(first_token) else {
            return Err(format!("缺少受保护占位符 {first_token}"));
        };
        parts.extend(distribute_to_fragment_range(
            record,
            entry,
            fragment_start,
            boundary,
            &remaining[..token_position],
        )?);
        remaining = &remaining[token_position..];
        for marker in &record.markers[marker_index..group_end] {
            if !remaining.starts_with(&marker.token) {
                return Err(format!(
                    "同一位置的受保护占位符之间不能插入文本；应为 {}",
                    marker.token
                ));
            }
            remaining = &remaining[marker.token.len()..];
        }
        fragment_start = boundary;
        marker_index = group_end;
    }
    parts.extend(distribute_to_fragment_range(
        record,
        entry,
        fragment_start,
        record.fragments.len(),
        remaining,
    )?);
    Ok(parts)
}

fn distribute_to_fragment_range(
    record: &ScriptTextRecord,
    entry: &vn_text::Entry,
    start: usize,
    end: usize,
    translated: &str,
) -> Result<Vec<(usize, String)>, String> {
    if start > end || end > record.fragments.len() {
        return Err("翻译片段定位超出记录范围".into());
    }
    let fragments = &record.fragments[start..end];
    if fragments.is_empty() {
        return if translated.is_empty() {
            Ok(Vec::new())
        } else {
            Err("两个连续受保护占位符之间无法注入文本".into())
        };
    }
    if fragments.len() == 1 {
        return Ok(vec![(fragments[0].offset, translated.to_owned())]);
    }

    let original_name_length = if start == 0 {
        record
            .name
            .as_deref()
            .map_or(0, |name| name.chars().count())
    } else {
        0
    };
    let original_total = fragments
        .iter()
        .map(|fragment| fragment.source_text.chars().count())
        .sum::<usize>();
    let original_body_length = original_total.saturating_sub(original_name_length);
    let translated_chars = translated.chars().collect::<Vec<_>>();
    let translated_name_length = if start == 0 {
        entry.name.as_deref().map_or(0, |name| name.chars().count())
    } else {
        0
    };
    if translated_name_length > translated_chars.len() {
        return Err("翻译后的姓名长度超过对应文本片段".into());
    }
    let translated_body_end = translated_chars.len();
    let translated_body_length = translated_body_end.saturating_sub(translated_name_length);

    let mut parts = Vec::with_capacity(fragments.len());
    let mut previous_new_boundary = 0usize;
    let mut original_boundary = 0usize;
    for (local_index, fragment) in fragments.iter().enumerate() {
        if local_index + 1 == fragments.len() {
            parts.push((
                fragment.offset,
                translated_chars[previous_new_boundary..]
                    .iter()
                    .collect::<String>(),
            ));
            break;
        }
        original_boundary += fragment.source_text.chars().count();
        let old_body_boundary = original_boundary
            .saturating_sub(original_name_length)
            .min(original_body_length);
        let new_body_boundary = if original_body_length == 0 {
            0
        } else {
            let scaled = (old_body_boundary as u128 * translated_body_length as u128
                + original_body_length as u128 / 2)
                / original_body_length as u128;
            usize::try_from(scaled).unwrap_or(translated_body_length)
        }
        .min(translated_body_length);
        let mut new_boundary = (translated_name_length + new_body_boundary)
            .max(previous_new_boundary)
            .min(translated_body_end);
        if start == 0 && local_index == 0 && entry.name.is_some() {
            new_boundary = new_boundary.max((translated_name_length + 1).min(translated_body_end));
        }
        if record.speaker_quote_closed && end == record.fragments.len() {
            new_boundary = new_boundary.min(translated_body_end.saturating_sub(1));
        }
        parts.push((
            fragment.offset,
            translated_chars[previous_new_boundary..new_boundary]
                .iter()
                .collect::<String>(),
        ));
        previous_new_boundary = new_boundary;
    }
    Ok(parts)
}

/// Build payload-relative replacement operands for extracted 2049/2112 strings.
///
/// The encoder returns CP932 bytes without a terminator. This function adds
/// the required NUL, rejects embedded NULs, and ensures the encoded operand
/// including its terminator fits the instruction's u16 byte-length field.
pub(super) fn build_replacements(
    records: &[ScriptTextRecord],
    entries: &[vn_text::Entry],
    encode: impl Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<BTreeMap<usize, Vec<u8>>, String> {
    let source_entries = translation_entries(records);
    vn_text::validate_shape(&source_entries, entries)
        .map_err(|error| format!("translation template shape mismatch: {error}"))?;

    let mut replacements = BTreeMap::new();
    for (index, (record, entry)) in records.iter().zip(entries).enumerate() {
        if !matches!(
            (record.kind, record.opcode),
            (RecordKind::Display, 2112) | (RecordKind::ChoiceOption, 2049)
        ) {
            return Err(format!(
                "translation record {} has unsupported opcode {}",
                index + 1,
                record.opcode
            ));
        }
        let text = translated_text(record, entry)
            .map_err(|error| format!("invalid translation entry {}: {error}", index + 1))?;
        for (offset, fragment_text) in distribute_translation(record, entry, &text)
            .map_err(|error| format!("invalid translation entry {}: {error}", index + 1))?
        {
            let mut encoded = encode(&fragment_text).map_err(|error| {
                format!("failed to encode translation entry {}: {error}", index + 1)
            })?;
            if encoded.contains(&0) {
                return Err(format!(
                    "translation entry {} encodes to bytes containing an internal NUL",
                    index + 1
                ));
            }
            let length_with_nul = encoded
                .len()
                .checked_add(1)
                .ok_or_else(|| format!("translation entry {} byte length overflow", index + 1))?;
            if length_with_nul > u16::MAX as usize {
                return Err(format!(
                    "translation entry {} fragment at 0x{offset:X} is {length_with_nul} bytes including NUL, exceeding the u16 string-length limit",
                    index + 1
                ));
            }
            encoded.push(0);
            if replacements.insert(offset, encoded).is_some() {
                return Err(format!("duplicate string replacement offset 0x{offset:X}"));
            }
        }
    }

    Ok(replacements)
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
        assert!(!had_errors, "test fixture must be representable in CP932");
        let mut bytes = encoded.into_owned();
        bytes.push(0);
        let offset = payload.len();
        push_opcode(payload, opcode);
        payload.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        payload.extend_from_slice(&bytes);
        offset
    }

    fn push_call(payload: &mut Vec<u8>, opcode: u16, hash: u32, argument_count: u16) {
        push_opcode(payload, opcode);
        payload.extend_from_slice(&hash.to_le_bytes());
        payload.extend_from_slice(&0u32.to_le_bytes());
        payload.extend_from_slice(&argument_count.to_le_bytes());
    }

    fn sample_payload() -> (Vec<u8>, Vec<usize>) {
        let mut payload = Vec::new();
        let mut text_offsets = Vec::new();
        text_offsets.push(push_text(&mut payload, 2112, "決して"));
        push_opcode(&mut payload, 2113);
        push_text(&mut payload, 2114, "n");
        push_opcode(&mut payload, 2106);
        payload.extend_from_slice(&0u16.to_le_bytes());
        text_offsets.push(push_text(&mut payload, 2112, "屈するつもりもない』"));
        push_opcode(&mut payload, 2113);

        text_offsets.push(push_text(&mut payload, 2112, "誰も居ない空間をジッと"));
        push_opcode(&mut payload, 2113);
        push_text(&mut payload, 2049, "にら");
        push_call(&mut payload, 2064, RUBY_HELPER_HASH, 1);
        text_offsets.push(push_text(
            &mut payload,
            2112,
            "睨みつけ、あたしは頭をゆっくりと左右に振る。",
        ));
        push_opcode(&mut payload, 2113);

        push_text(&mut payload, 2049, "「ひとりでお留守番……できるかな？」");
        push_text(&mut payload, 2049, "「ふたりでミルクを取りに行くかい？」");
        push_call(&mut payload, 2063, CHOICE_HELPER_HASH, 2);
        push_opcode(&mut payload, 2096);
        payload.extend_from_slice(&0i32.to_le_bytes());
        push_opcode(&mut payload, 2091);
        (payload, text_offsets)
    }

    fn parse_sample(payload: &[u8]) -> BytecodeProgram {
        crate::mjo_bytecode::parse_payload(payload, &[(1, 0)], 0).unwrap()
    }

    #[test]
    fn retains_newline_ruby_choices_and_their_source_links() {
        let (payload, _) = sample_payload();
        let program = parse_sample(&payload);
        let report = extract_records_with_report(&payload, &program).unwrap();

        let newline = report
            .records
            .iter()
            .find(|record| record.message.contains("決して"))
            .unwrap();
        assert_eq!(
            newline.message,
            format!("決して{LINE_BREAK_TOKEN}屈するつもりもない』")
        );
        assert!(newline
            .markers
            .iter()
            .any(|marker| marker.kind == MarkerKind::LineBreak));

        let ruby = report
            .records
            .iter()
            .find(|record| record.message.contains("睨みつけ"))
            .unwrap();
        assert!(ruby.message.contains("〔注音:にら〕"));
        assert_eq!(ruby.fragments.len(), 2);
        let ruby_marker = ruby
            .markers
            .iter()
            .find(|marker| marker.kind == MarkerKind::Ruby)
            .unwrap();
        assert_eq!(ruby_marker.source_text.as_deref(), Some("にら"));
        assert_eq!(
            ruby_marker.ruby_target_offset,
            Some(ruby.fragments[1].offset)
        );
        assert_eq!(ruby_marker.ruby_start_char, Some(0));
        assert_eq!(ruby_marker.ruby_length_chars, Some(1));

        let choices = report
            .records
            .iter()
            .filter(|record| record.kind == RecordKind::ChoiceOption)
            .collect::<Vec<_>>();
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0].message, "ふたりでミルクを取りに行くかい？");
        assert_eq!(choices[1].message, "ひとりでお留守番……できるかな？");
        assert!(choices.iter().all(|record| {
            record.choice_control.as_ref().is_some_and(|control| {
                control.callee_hash == CHOICE_HELPER_HASH
                    && control.argument_count == 2
                    && control.option_offsets.len() == 2
                    && !control.flow_edges.is_empty()
            })
        }));
    }

    #[test]
    fn unchanged_and_edited_entries_only_replace_their_own_text_operands() {
        let (payload, text_offsets) = sample_payload();
        let program = parse_sample(&payload);
        let report = extract_records_with_report(&payload, &program).unwrap();
        let mut entries = translation_entries(&report.records);
        entries[0].message = format!("決して{LINE_BREAK_TOKEN}ひるまない』");
        let choices_start = entries
            .iter()
            .position(|entry| entry.message == "ひとりでお留守番……できるかな？")
            .unwrap();
        entries[choices_start].message = "ひとりで待っていてくれる？".into();

        let replacements = build_replacements(&report.records, &entries, |text| {
            let (encoded, _, had_errors) = encoding_rs::SHIFT_JIS.encode(text);
            if had_errors {
                Err("not representable".into())
            } else {
                Ok(encoded.into_owned())
            }
        })
        .unwrap();
        for offset in text_offsets {
            assert!(replacements.contains_key(&offset));
        }
        let choice = report
            .records
            .iter()
            .zip(&entries)
            .find(|(record, entry)| {
                record.kind == RecordKind::ChoiceOption
                    && entry.message == "ひとりで待っていてくれる？"
            })
            .unwrap();
        let rebuilt_option = translated_text(choice.0, choice.1).unwrap();
        assert_eq!(rebuilt_option, "「ひとりで待っていてくれる？」");
    }
}
