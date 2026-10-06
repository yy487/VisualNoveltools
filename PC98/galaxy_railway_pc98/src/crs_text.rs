//! Reachable-script text extraction and append-and-repoint rebuilding for CRS files.
//!
//! The script walker models the verified VM control transfers instead of
//! scanning for display-opcode byte signatures. Strings referenced by reachable
//! 0x53/0x79 commands are exported once per distinct pointer. A reached 0x47 is
//! an external file load and ends that path; any independently supplied script
//! is handled from its own header by the caller.

use crate::{font_plan::encode_display, sha256, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use vn_font::font_98::EncodingPlan;

pub const CRS_SCHEMA: &str = "galaxy-railway-pc98-crs-v1";
const ENCODING: &str = "CP932";
const MAX_CRS_SIZE: usize = 0x7FFF;
const MAX_CALL_DEPTH: usize = 30;
const MAX_WORK_STATES: usize = 1_000_000;

/// One hash-bound translation document for a CRS script.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrsDocument {
    pub schema: String,
    pub source_file: String,
    pub source_sha256: String,
    pub encoding: String,
    pub entries: Vec<CrsEntry>,
    pub diagnostics: Vec<String>,
}

/// A distinct display-string pointer and every reachable instruction operand
/// that refers to it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CrsEntry {
    #[serde(rename = "_index")]
    pub index: usize,
    /// Absolute byte pointer to the source string.
    #[serde(rename = "_offset")]
    pub offset: usize,
    /// Number of source CP932 bytes, excluding the NUL terminator.
    #[serde(rename = "_byte_length")]
    pub byte_length: usize,
    /// Byte offsets of the two-byte F1 pointer values in the script.
    #[serde(rename = "_reference_offsets")]
    pub reference_offsets: Vec<usize>,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Next,
    Call(usize),
    Return,
    Jump(usize),
    Conditional(usize),
    ExternalLoad,
    Stop,
}

#[derive(Debug, Clone)]
struct Instruction {
    length: usize,
    flow: Flow,
    display_pointer: Option<usize>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct State {
    pc: usize,
    return_stack: Vec<usize>,
}

#[derive(Debug)]
struct Program {
    entry_pc: usize,
    instructions: BTreeMap<usize, Instruction>,
    byte_owner: Vec<Option<usize>>,
    stop_offsets: BTreeSet<usize>,
    external_loads: BTreeSet<usize>,
    empty_returns: BTreeSet<usize>,
}

fn read_u16(raw: &[u8], offset: usize, what: &str) -> Result<u16> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| format!("{what} offset overflow"))?;
    let bytes = raw
        .get(offset..end)
        .ok_or_else(|| format!("{what} is truncated at 0x{offset:X}"))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn write_u16(raw: &mut [u8], offset: usize, value: u16, what: &str) -> Result<()> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| format!("{what} offset overflow"))?;
    let bytes = raw
        .get_mut(offset..end)
        .ok_or_else(|| format!("{what} is outside the CRS source"))?;
    bytes.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn strict_cp932_decode(raw: &[u8]) -> Option<String> {
    let decoded =
        encoding_rs::SHIFT_JIS.decode_without_bom_handling_and_without_replacement(raw)?;
    let (encoded, _, had_errors) = encoding_rs::SHIFT_JIS.encode(&decoded);
    if had_errors || encoded.as_ref() != raw {
        return None;
    }
    Some(decoded.into_owned())
}

fn static_operand_count(opcode: u8) -> Option<usize> {
    Some(match opcode {
        0x20 => 2,
        0x21 | 0x22 | 0x29 | 0x2A | 0x2E | 0x35 | 0x3A | 0x40 | 0x44 | 0x46 | 0x4F | 0x57
        | 0x58 | 0x5E | 0x5F | 0x63 | 0x66 | 0x6B | 0x72 | 0x73 | 0x74 | 0x75 => 1,
        0x23 | 0x25 | 0x27 | 0x28 | 0x71 => 6,
        0x24 | 0x5B => 8,
        0x26 | 0x2B | 0x41 | 0x48 | 0x5C | 0x67 | 0x76 => 2,
        0x30 | 0x68 | 0x6D | 0x77 | 0x78 => 3,
        0x31 | 0x39 | 0x42 => 4,
        0x32 | 0x3D | 0x3F | 0x55 | 0x59 | 0x62 => 5,
        0x33 | 0x43 => 3,
        0x36 | 0x49 | 0x4D | 0x60 | 0x61 | 0x64 | 0x65 | 0x6E | 0x6F => 1,
        0x37 | 0x3B | 0x3E | 0x5D => 3,
        0x3C | 0x56 => 2,
        0x47 => 1,
        0x51 | 0x52 | 0x54 => 5,
        0x53 | 0x79 => 4,
        0x69 => 7,
        0x6C | 0x70 | 0x7A | 0x7B => 1,
        0x2C | 0x2D | 0x2F | 0x34 | 0x45 | 0x4E | 0x50 | 0x7C | 0x7D => 0,
        _ => return None,
    })
}

fn is_known_opcode(opcode: u8) -> bool {
    matches!(
        opcode,
        0x16 | 0x17 | 0x18 | 0x19 | 0x1A | 0x1B | 0x1C | 0x1E | 0xEC | 0xEE | 0xEF | 0xFF
    ) || static_operand_count(opcode).is_some()
}

fn ensure_range(raw: &[u8], start: usize, length: usize, what: &str) -> Result<usize> {
    let end = start
        .checked_add(length)
        .ok_or_else(|| format!("{what} length overflows at 0x{start:X}"))?;
    if end > raw.len() {
        return Err(format!("{what} is truncated at 0x{start:X}"));
    }
    Ok(end)
}

fn parse_expression(
    raw: &[u8],
    start: usize,
    allow_conditions: bool,
) -> Result<(usize, Option<usize>)> {
    let mut cursor = start;
    let mut saw_condition = false;
    loop {
        let token = *raw
            .get(cursor)
            .ok_or_else(|| format!("expression is unterminated at 0x{start:X}"))?;
        match token {
            0xF0..=0xF4 => {
                cursor = ensure_range(raw, cursor, 3, "typed expression operand")?;
            }
            0x01 | 0x02 | 0x03 | 0x04 | 0x08 => cursor += 1,
            0x05 if !allow_conditions => return Ok((cursor + 1, None)),
            0x05 | 0x06 | 0x07 | 0x09 if allow_conditions => {
                saw_condition = true;
                cursor += 1;
            }
            0xED if allow_conditions && saw_condition => {
                let target = usize::from(read_u16(raw, cursor + 1, "EC branch target")?);
                let end = ensure_range(raw, cursor, 3, "EC branch target")?;
                return Ok((end, Some(target)));
            }
            _ => {
                let mode = if allow_conditions {
                    "EC condition"
                } else {
                    "FF expression"
                };
                return Err(format!(
                    "unsupported {mode} token 0x{token:02X} at 0x{cursor:X}"
                ));
            }
        }
    }
}

fn parse_instruction(raw: &[u8], pc: usize) -> Result<Instruction> {
    let opcode = *raw
        .get(pc)
        .ok_or_else(|| format!("program counter 0x{pc:X} is past CRS EOF"))?;
    match opcode {
        0x1E => Ok(Instruction {
            length: 1,
            flow: Flow::Stop,
            display_pointer: None,
        }),
        0x16 | 0x19 | 0xEE => {
            let target = usize::from(read_u16(raw, pc + 1, "absolute CRS target")?);
            let end = ensure_range(raw, pc, 3, "absolute CRS transfer")?;
            let flow = match opcode {
                0x16 => Flow::Call(target),
                _ => Flow::Jump(target),
            };
            Ok(Instruction {
                length: end - pc,
                flow,
                display_pointer: None,
            })
        }
        0x17 => Ok(Instruction {
            length: 1,
            flow: Flow::Return,
            display_pointer: None,
        }),
        0xEC => {
            ensure_range(raw, pc, 2, "EC mode byte")?;
            let (end, target) = parse_expression(raw, pc + 2, true)?;
            let target = target.ok_or_else(|| format!("EC at 0x{pc:X} has no branch target"))?;
            Ok(Instruction {
                length: end - pc,
                flow: Flow::Conditional(target),
                display_pointer: None,
            })
        }
        0xFF => {
            let (end, _) = parse_expression(raw, pc + 1, false)?;
            Ok(Instruction {
                length: end - pc,
                flow: Flow::Next,
                display_pointer: None,
            })
        }
        0x47 | 0x53 | 0x79 => parse_typed_instruction(raw, pc, opcode),
        _ => {
            if matches!(opcode, 0x18 | 0xEF) {
                return Ok(Instruction {
                    length: 1,
                    flow: Flow::Next,
                    display_pointer: None,
                });
            }
            if matches!(opcode, 0x1A..=0x1C) {
                ensure_range(raw, pc, 3, "state command")?;
                return Ok(Instruction {
                    length: 3,
                    flow: Flow::Next,
                    display_pointer: None,
                });
            }
            parse_typed_instruction(raw, pc, opcode)
        }
    }
}

fn parse_typed_instruction(raw: &[u8], pc: usize, opcode: u8) -> Result<Instruction> {
    let count = static_operand_count(opcode)
        .ok_or_else(|| format!("unsupported CRS opcode 0x{opcode:02X} at 0x{pc:X}"))?;
    let length = 1usize
        .checked_add(count.checked_mul(3).ok_or("typed operand count overflow")?)
        .ok_or("typed instruction length overflow")?;
    ensure_range(raw, pc, length, "typed CRS instruction")?;
    let mut tokens = Vec::with_capacity(count);
    for index in 0..count {
        let token_offset = pc + 1 + index * 3;
        let token = raw[token_offset];
        if !(0xF0..=0xF4).contains(&token) {
            return Err(format!(
                "opcode 0x{opcode:02X} has invalid typed operand 0x{token:02X} at 0x{token_offset:X}"
            ));
        }
        tokens.push(token);
    }

    if opcode == 0x47 && !matches!(tokens[0], 0xF1 | 0xF4) {
        return Err(format!(
            "0x47 at 0x{pc:X} has a filename operand with an unsupported value type"
        ));
    }
    if matches!(opcode, 0x53 | 0x79) {
        if !matches!(tokens[0], 0xF0 | 0xF3)
            || !matches!(tokens[1], 0xF0 | 0xF3)
            || tokens[2] != 0xF1
            || !matches!(tokens[3], 0xF0 | 0xF3)
        {
            return Err(format!(
                "display opcode 0x{opcode:02X} at 0x{pc:X} has unsupported typed operands"
            ));
        }
        let pointer = usize::from(read_u16(raw, pc + 8, "display string pointer")?);
        return Ok(Instruction {
            length,
            flow: Flow::Next,
            display_pointer: Some(pointer),
        });
    }
    if matches!(opcode, 0x51 | 0x52 | 0x54)
        && (!matches!(tokens[0], 0xF0 | 0xF3) || tokens[1..].iter().any(|token| *token != 0xF3))
    {
        return Err(format!(
            "opcode 0x{opcode:02X} at 0x{pc:X} has unsupported typed operands"
        ));
    }

    Ok(Instruction {
        length,
        flow: if opcode == 0x47 {
            Flow::ExternalLoad
        } else {
            Flow::Next
        },
        display_pointer: None,
    })
}

fn validate_instruction_span(
    program: &mut Program,
    pc: usize,
    instruction: &Instruction,
) -> Result<()> {
    let end = pc
        .checked_add(instruction.length)
        .ok_or_else(|| format!("instruction at 0x{pc:X} overflows its end"))?;
    if end > program.byte_owner.len() {
        return Err(format!("instruction at 0x{pc:X} extends past CRS EOF"));
    }
    for offset in pc..end {
        if let Some(owner) = program.byte_owner[offset] {
            if owner != pc {
                return Err(format!(
                    "instruction at 0x{pc:X} overlaps instruction at 0x{owner:X}"
                ));
            }
        }
        if offset != pc && program.stop_offsets.contains(&offset) {
            return Err(format!(
                "instruction at 0x{pc:X} overlaps reached stop byte at 0x{offset:X}"
            ));
        }
    }
    for offset in pc..end {
        program.byte_owner[offset] = Some(pc);
    }
    Ok(())
}

fn enqueue_state(queue: &mut VecDeque<State>, state: State, visited: &HashSet<State>) {
    if !visited.contains(&state) {
        queue.push_back(state);
    }
}

fn parse_program(raw: &[u8], entry_pc: usize) -> Result<Program> {
    let mut program = Program {
        entry_pc,
        instructions: BTreeMap::new(),
        byte_owner: vec![None; raw.len()],
        stop_offsets: BTreeSet::new(),
        external_loads: BTreeSet::new(),
        empty_returns: BTreeSet::new(),
    };
    let mut queue = VecDeque::from([State {
        pc: entry_pc,
        return_stack: Vec::new(),
    }]);
    let mut visited = HashSet::new();

    while let Some(state) = queue.pop_front() {
        if !visited.insert(state.clone()) {
            continue;
        }
        if visited.len() > MAX_WORK_STATES {
            return Err(format!(
                "CRS control-flow walk exceeds {MAX_WORK_STATES} states"
            ));
        }
        if state.pc >= raw.len() {
            return Err(format!("program counter 0x{:X} is past CRS EOF", state.pc));
        }
        if raw[state.pc] == 0x1E {
            if let Some(owner) = program.byte_owner[state.pc] {
                if owner != state.pc {
                    return Err(format!(
                        "stop byte at 0x{:X} overlaps instruction at 0x{owner:X}",
                        state.pc
                    ));
                }
            }
            program.stop_offsets.insert(state.pc);
            continue;
        }

        let instruction = if let Some(instruction) = program.instructions.get(&state.pc) {
            instruction.clone()
        } else {
            let instruction = parse_instruction(raw, state.pc)?;
            validate_instruction_span(&mut program, state.pc, &instruction)?;
            program.instructions.insert(state.pc, instruction.clone());
            instruction
        };
        let next_pc = state
            .pc
            .checked_add(instruction.length)
            .ok_or_else(|| format!("next PC overflows at 0x{:X}", state.pc))?;

        match instruction.flow {
            Flow::Next => enqueue_state(
                &mut queue,
                State {
                    pc: next_pc,
                    return_stack: state.return_stack,
                },
                &visited,
            ),
            Flow::Call(target) => {
                if state.return_stack.len() >= MAX_CALL_DEPTH {
                    return Err(format!(
                        "0x16 call at 0x{:X} exceeds the {MAX_CALL_DEPTH}-entry return stack",
                        state.pc
                    ));
                }
                let mut return_stack = state.return_stack;
                return_stack.push(next_pc);
                enqueue_state(
                    &mut queue,
                    State {
                        pc: target,
                        return_stack,
                    },
                    &visited,
                );
            }
            Flow::Return => {
                let mut return_stack = state.return_stack;
                if let Some(return_pc) = return_stack.pop() {
                    enqueue_state(
                        &mut queue,
                        State {
                            pc: return_pc,
                            return_stack,
                        },
                        &visited,
                    );
                } else {
                    // The VM raises its return-stack error and returns the
                    // FFFF sentinel; the outer loop treats that as terminal.
                    program.empty_returns.insert(state.pc);
                }
            }
            Flow::Jump(target) => enqueue_state(
                &mut queue,
                State {
                    pc: target,
                    return_stack: state.return_stack,
                },
                &visited,
            ),
            Flow::Conditional(target) => {
                enqueue_state(
                    &mut queue,
                    State {
                        pc: next_pc,
                        return_stack: state.return_stack.clone(),
                    },
                    &visited,
                );
                enqueue_state(
                    &mut queue,
                    State {
                        pc: target,
                        return_stack: state.return_stack,
                    },
                    &visited,
                );
            }
            Flow::ExternalLoad => {
                program.external_loads.insert(state.pc);
            }
            Flow::Stop => {
                program.stop_offsets.insert(state.pc);
            }
        }
    }

    for (pc, instruction) in &program.instructions {
        let next_pc = pc + instruction.length;
        let targets = match instruction.flow {
            Flow::Call(target) | Flow::Jump(target) | Flow::Conditional(target) => vec![target],
            _ => Vec::new(),
        };
        for target in targets {
            if !program.instructions.contains_key(&target)
                && !program.stop_offsets.contains(&target)
            {
                return Err(format!(
                    "control transfer at 0x{pc:X} targets unresolved offset 0x{target:X}"
                ));
            }
        }
        if matches!(instruction.flow, Flow::Next | Flow::Conditional(_))
            && next_pc < raw.len()
            && raw[next_pc] == 0x1E
            && !program.stop_offsets.contains(&next_pc)
        {
            return Err(format!("stop successor 0x{next_pc:X} was not reached"));
        }
    }
    Ok(program)
}

fn parse_crs(raw: &[u8]) -> Result<Option<(Program, usize)>> {
    if raw.len() > MAX_CRS_SIZE {
        return Err(format!(
            "CRS source is {} bytes; maximum supported size is {MAX_CRS_SIZE}",
            raw.len()
        ));
    }
    if raw.len() < 2 {
        return Err("CRS source is shorter than its 2-byte entry-point header".into());
    }
    let entry_pc = usize::from(read_u16(raw, 0, "CRS entry point")?);
    if entry_pc < 2 || entry_pc >= raw.len() || !is_known_opcode(raw[entry_pc]) {
        return Ok(None);
    }
    let program = parse_program(raw, entry_pc)?;
    Ok(Some((program, entry_pc)))
}

fn make_document(raw: &[u8], source_file: String) -> Result<Option<CrsDocument>> {
    let Some((program, entry_pc)) = parse_crs(raw)? else {
        return Ok(None);
    };
    let mut references: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (pc, instruction) in &program.instructions {
        if let Some(pointer) = instruction.display_pointer {
            references.entry(pointer).or_default().push(pc + 8);
        }
    }

    let mut entries = Vec::with_capacity(references.len());
    for (index, (pointer, mut reference_offsets)) in references.into_iter().enumerate() {
        if pointer < 2 || pointer >= raw.len() {
            return Err(format!(
                "display string pointer 0x{pointer:X} is outside the CRS source"
            ));
        }
        let nul_offset = raw[pointer..]
            .iter()
            .position(|byte| *byte == 0)
            .map(|relative| pointer + relative)
            .ok_or_else(|| format!("display string at 0x{pointer:X} is not NUL-terminated"))?;
        if (pointer..=nul_offset).any(|offset| {
            program.byte_owner[offset].is_some() || program.stop_offsets.contains(&offset)
        }) {
            return Err(format!(
                "display string at 0x{pointer:X} overlaps a reachable code boundary"
            ));
        }
        let bytes = &raw[pointer..nul_offset];
        let scr_msg = strict_cp932_decode(bytes)
            .ok_or_else(|| format!("display string at 0x{pointer:X} is not reversible CP932"))?;
        reference_offsets.sort_unstable();
        entries.push(CrsEntry {
            index,
            offset: pointer,
            byte_length: bytes.len(),
            reference_offsets,
            message: scr_msg.clone(),
            scr_msg,
        });
    }

    let mut diagnostics: Vec<String> = program
        .external_loads
        .iter()
        .map(|pc| {
            format!("reachable 0x47 at 0x{pc:X} transfers execution to an externally loaded file")
        })
        .collect();
    diagnostics.extend(program.empty_returns.iter().map(|pc| {
        format!(
            "reachable 0x17 at 0x{pc:X} returns with an empty stack and terminates via the VM error sentinel"
        )
    }));
    debug_assert!(program.entry_pc == entry_pc);
    Ok(Some(CrsDocument {
        schema: CRS_SCHEMA.to_owned(),
        source_file,
        source_sha256: sha256(raw),
        encoding: ENCODING.to_owned(),
        entries,
        diagnostics,
    }))
}

/// Extract one document from a CRS script. `Ok(None)` identifies an input whose
/// header does not point to a supported script opcode; malformed reachable
/// script data returns an error.
pub fn extract_crs(raw: &[u8], source_file: impl Into<String>) -> Result<Option<CrsDocument>> {
    make_document(raw, source_file.into())
}

fn validate_document(supplied: &CrsDocument, expected: &CrsDocument) -> Result<()> {
    if supplied.schema != expected.schema
        || supplied.source_file != expected.source_file
        || supplied.source_sha256 != expected.source_sha256
        || supplied.encoding != ENCODING
        || supplied.diagnostics != expected.diagnostics
        || supplied.entries.len() != expected.entries.len()
    {
        return Err(format!(
            "CRS translation metadata, diagnostics, or entry count does not match {}",
            expected.source_file
        ));
    }
    for (actual, baseline) in supplied.entries.iter().zip(&expected.entries) {
        if actual.index != baseline.index
            || actual.offset != baseline.offset
            || actual.byte_length != baseline.byte_length
            || actual.reference_offsets != baseline.reference_offsets
            || actual.scr_msg != baseline.scr_msg
        {
            return Err(format!(
                "CRS entry {} original text or locator metadata was modified",
                baseline.index
            ));
        }
        if actual.message != baseline.scr_msg && actual.message.chars().any(char::is_control) {
            return Err(format!(
                "{} entry {} contains a control character; CRS line-break behavior is not verified",
                expected.source_file, baseline.index
            ));
        }
    }
    Ok(())
}

/// Rebuild a CRS by appending each changed string and repointing all of its
/// reachable F1 operands. Unchanged documents return the original bytes exactly.
pub fn apply_crs(raw: &[u8], document: &CrsDocument, encoding: &EncodingPlan) -> Result<Vec<u8>> {
    let expected = make_document(raw, document.source_file.clone())?
        .ok_or_else(|| format!("{} is not a supported CRS script", document.source_file))?;
    validate_document(document, &expected)?;
    if document
        .entries
        .iter()
        .all(|entry| entry.message == entry.scr_msg)
    {
        return Ok(raw.to_vec());
    }

    let mut rebuilt = raw.to_vec();
    for entry in &document.entries {
        if entry.message == entry.scr_msg {
            continue;
        }
        let encoded = encode_display(&entry.message, encoding).map_err(|error| {
            format!(
                "{} entry {} cannot be encoded: {error}",
                document.source_file, entry.index
            )
        })?;
        let new_offset = rebuilt.len();
        let pointer = u16::try_from(new_offset).map_err(|_| {
            format!(
                "{} entry {} append offset exceeds a 16-bit pointer",
                document.source_file, entry.index
            )
        })?;
        let new_len = new_offset
            .checked_add(encoded.len())
            .and_then(|end| end.checked_add(1))
            .ok_or_else(|| format!("{} rebuilt size overflow", document.source_file))?;
        if new_len > MAX_CRS_SIZE {
            return Err(format!(
                "{} rebuilt CRS would exceed {MAX_CRS_SIZE} bytes",
                document.source_file
            ));
        }
        for &reference_offset in &entry.reference_offsets {
            write_u16(
                &mut rebuilt,
                reference_offset,
                pointer,
                "display string reference",
            )?;
        }
        rebuilt.extend_from_slice(&encoded);
        rebuilt.push(0);
    }
    Ok(rebuilt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use encoding_rs::SHIFT_JIS;
    use vn_font::font_98::{EncodingPlan, SubstitutionMap};

    fn cp932(text: &str) -> Vec<u8> {
        let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
        assert!(!had_errors);
        encoded.into_owned()
    }

    fn display(pointer: u16) -> [u8; 13] {
        [
            0x53,
            0xF0,
            0,
            0,
            0xF0,
            0,
            0,
            0xF1,
            pointer as u8,
            (pointer >> 8) as u8,
            0xF0,
            0,
            0,
        ]
    }

    fn synthetic_crs() -> Vec<u8> {
        let mut raw = vec![0xCC; 0x70];
        raw[..2].copy_from_slice(&0x20u16.to_le_bytes());
        let text = cp932("客室");
        raw[0x10..0x10 + text.len()].copy_from_slice(&text);
        raw[0x10 + text.len()] = 0;

        // Main flow calls a subroutine and reaches its 1E stop after return.
        raw[0x20..0x23].copy_from_slice(&[0x16, 0x40, 0x00]);
        raw[0x23] = 0x1E;
        raw[0x40..0x4D].copy_from_slice(&display(0x10));
        raw[0x4D..0x5A].copy_from_slice(&display(0x10));
        raw[0x5A] = 0x17;
        raw
    }

    fn plan(texts: &[&str]) -> EncodingPlan {
        let substitutions = SubstitutionMap::embedded().expect("embedded substitutions");
        EncodingPlan::build(
            &substitutions,
            std::iter::empty::<u16>(),
            texts.iter().copied(),
        )
        .expect("synthetic encoding plan")
    }

    #[test]
    fn reachable_call_groups_duplicate_pointers_and_preserves_no_change_bytes() {
        let source = synthetic_crs();
        let document = extract_crs(&source, "HAK1.CRS")
            .unwrap()
            .expect("synthetic CRS document");
        assert_eq!(document.schema, CRS_SCHEMA);
        assert_eq!(document.encoding, ENCODING);
        assert_eq!(document.entries.len(), 1);
        assert_eq!(document.entries[0].scr_msg, "客室");
        assert_eq!(document.entries[0].offset, 0x10);
        assert_eq!(document.entries[0].byte_length, cp932("客室").len());
        assert_eq!(document.entries[0].reference_offsets, [0x48, 0x55]);
        assert_eq!(
            apply_crs(&source, &document, &plan(&["客室"])).unwrap(),
            source
        );
    }

    #[test]
    fn changed_text_appends_once_and_repoints_every_reference() {
        let source = synthetic_crs();
        let mut document = extract_crs(&source, "HAK1.CRS")
            .unwrap()
            .expect("synthetic CRS document");
        document.entries[0].message = "長い翻訳文".into();
        let rebuilt =
            apply_crs(&source, &document, &plan(&["長い翻訳文"])).expect("append-and-repoint CRS");
        let appended = source.len();
        assert_eq!(rebuilt.len(), appended + cp932("長い翻訳文").len() + 1);
        assert_eq!(
            read_u16(&rebuilt, 0x48, "first display pointer").unwrap() as usize,
            appended
        );
        assert_eq!(
            read_u16(&rebuilt, 0x55, "second display pointer").unwrap() as usize,
            appended
        );
        assert_eq!(&rebuilt[appended..rebuilt.len() - 1], cp932("長い翻訳文"));
        assert_eq!(rebuilt.last(), Some(&0));

        let reread = extract_crs(&rebuilt, "HAK1.CRS")
            .unwrap()
            .expect("rebuilt CRS document");
        assert_eq!(reread.entries[0].scr_msg, "長い翻訳文");
        assert_eq!(reread.entries[0].reference_offsets, [0x48, 0x55]);
    }

    #[test]
    fn stops_at_reached_1e_and_rejects_unverified_line_breaks_and_stale_hashes() {
        let mut source = synthetic_crs();
        // A signature-like display command after the reached stop is ignored.
        source[0x24..0x31].copy_from_slice(&display(0x10));
        let mut document = extract_crs(&source, "HAK1.CRS")
            .unwrap()
            .expect("synthetic CRS document");
        assert_eq!(document.entries[0].reference_offsets, [0x48, 0x55]);

        document.entries[0].message = "第一行\n第二行".into();
        assert!(apply_crs(&source, &document, &plan(&["第一行", "第二行"])).is_err());

        let mut stale = extract_crs(&source, "HAK1.CRS")
            .unwrap()
            .expect("synthetic CRS document");
        stale.source_sha256 = "00".repeat(32);
        assert!(apply_crs(&source, &stale, &plan(&["客室"])).is_err());
    }

    #[test]
    fn rejects_display_string_span_that_touches_a_reached_stop_byte() {
        let mut source = synthetic_crs();
        source[0x24] = 0;
        source[0x48..0x4A].copy_from_slice(&0x23u16.to_le_bytes());
        assert!(extract_crs(&source, "HAK1.CRS").is_err());
    }

    #[test]
    fn empty_return_stack_is_a_terminal_vm_error_path() {
        let mut source = vec![0; 0x30];
        source[..2].copy_from_slice(&0x20u16.to_le_bytes());
        source[0x20] = 0x17;
        source[0x21] = 0x1E;
        let document = extract_crs(&source, "RET.CRS")
            .unwrap()
            .expect("supported CRS document");
        assert!(document.entries.is_empty());
        assert!(document
            .diagnostics
            .iter()
            .any(|line| line.contains("empty stack")));
    }
}
