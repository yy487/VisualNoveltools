//! Clean MJO rebuilding with explicit instruction deletion and relocation.

use std::collections::{BTreeMap, BTreeSet};

use super::mjo_bytecode::{parse_payload, BytecodeProgram, RelocationKind};

const RUBY_HELPER_HASH: u32 = 0x3198_FD01;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RebuiltPayload {
    pub(super) payload: Vec<u8>,
    pub(super) symbols: Vec<(u32, u32)>,
    pub(super) entrypoint: u32,
    pub(super) offset_map: BTreeMap<usize, usize>,
}

pub(super) fn rebuild_payload(
    payload: &[u8],
    program: &BytecodeProgram,
    replacements: &BTreeMap<usize, Vec<u8>>,
    removals: &BTreeSet<usize>,
    symbols: &[(u32, u32)],
    entrypoint: u32,
) -> Result<RebuiltPayload, String> {
    let verified = parse_payload(payload, symbols, entrypoint)
        .map_err(|error| format!("rebuild source no longer parses: {error}"))?;
    if &verified != program {
        return Err("rebuild parser snapshot does not match the source payload".into());
    }

    let text_instructions = program
        .instructions
        .iter()
        .filter(|instruction| matches!(instruction.opcode, 2049 | 2112))
        .map(|instruction| (instruction.offset, instruction))
        .collect::<BTreeMap<_, _>>();
    for (&offset, replacement) in replacements {
        let instruction = text_instructions
            .get(&offset)
            .ok_or_else(|| format!("replacement 0x{offset:X} is not a text operand"))?;
        if removals.contains(&offset) {
            return Err(format!(
                "instruction 0x{offset:X} cannot be replaced and removed"
            ));
        }
        validate_operand(instruction.offset, replacement)?;
    }

    let instruction_by_offset = program
        .instructions
        .iter()
        .map(|instruction| (instruction.offset, instruction))
        .collect::<BTreeMap<_, _>>();
    for &offset in removals {
        let instruction = instruction_by_offset
            .get(&offset)
            .ok_or_else(|| format!("removal offset 0x{offset:X} is not an instruction"))?;
        let allowed = match instruction.opcode {
            2114 => instruction.text_bytes.as_deref() == Some(b"n\0"),
            2113 => {
                let index = program
                    .instructions
                    .binary_search_by_key(&offset, |candidate| candidate.offset)
                    .ok();
                index.is_some_and(|index| {
                    let next = program.instructions.get(index + 1);
                    next.is_some_and(|next| {
                        (is_linebreak(next) && removals.contains(&next.offset))
                            || (next.opcode == 2049
                                && program.instructions.get(index + 2).is_some_and(|helper| {
                                    helper.opcode == 2064
                                        && helper.script_call_hash == Some(RUBY_HELPER_HASH)
                                        && helper.script_call_argument_count == Some(1)
                                        && removals.contains(&next.offset)
                                        && removals.contains(&helper.offset)
                                        && program
                                            .instructions
                                            .get(index + 3)
                                            .is_some_and(|text| text.opcode == 2112)
                                }))
                    })
                })
            }
            2049 => program
                .instructions
                .binary_search_by_key(&offset, |candidate| candidate.offset)
                .ok()
                .and_then(|index| program.instructions.get(index + 1))
                .is_some_and(|helper| {
                    helper.opcode == 2064
                        && helper.script_call_hash == Some(RUBY_HELPER_HASH)
                        && helper.script_call_argument_count == Some(1)
                }),
            2064 => {
                instruction.script_call_hash == Some(RUBY_HELPER_HASH)
                    && instruction.script_call_argument_count == Some(1)
                    && program
                        .instructions
                        .binary_search_by_key(&offset, |i| i.offset)
                        .ok()
                        .and_then(|index| index.checked_sub(1))
                        .is_some_and(|index| {
                            let reading = &program.instructions[index];
                            reading.opcode == 2049 && removals.contains(&reading.offset)
                        })
            }
            _ => false,
        };
        if !allowed {
            return Err(format!(
                "refusing to remove unconfirmed opcode {} at 0x{offset:X}",
                instruction.opcode
            ));
        }
    }

    let mut rebuilt = Vec::with_capacity(payload.len());
    let mut offset_map = BTreeMap::new();
    let mut instruction_new_offsets = BTreeMap::new();
    for instruction in &program.instructions {
        let old_start = instruction.offset;
        let old_end = old_start
            .checked_add(instruction.size)
            .ok_or_else(|| format!("instruction 0x{old_start:X} end overflows"))?;
        let new_start = rebuilt.len();
        insert_boundary(&mut offset_map, old_start, new_start)?;
        instruction_new_offsets.insert(old_start, new_start);
        let source = payload
            .get(old_start..old_end)
            .ok_or_else(|| format!("instruction 0x{old_start:X} exceeds source"))?;
        if removals.contains(&old_start) {
            insert_boundary(&mut offset_map, old_end, rebuilt.len())?;
            continue;
        }
        if matches!(instruction.opcode, 2049 | 2112) {
            let original = instruction
                .text_bytes
                .as_deref()
                .ok_or_else(|| format!("text operand at 0x{old_start:X} was not parsed"))?;
            let operand = replacements.get(&old_start).map_or(original, Vec::as_slice);
            let length = u16::try_from(operand.len())
                .map_err(|_| format!("text operand at 0x{old_start:X} exceeds u16"))?;
            rebuilt.extend_from_slice(&source[..2]);
            rebuilt.extend_from_slice(&length.to_le_bytes());
            rebuilt.extend_from_slice(operand);
        } else {
            rebuilt.extend_from_slice(source);
        }
        insert_boundary(&mut offset_map, old_end, rebuilt.len())?;
    }
    if program
        .instructions
        .last()
        .is_some_and(|last| last.offset + last.size != payload.len())
    {
        return Err("decoded instruction stream does not cover the MJO payload".into());
    }
    insert_boundary(&mut offset_map, payload.len(), rebuilt.len())?;

    for relocation in &program.relocations {
        if removals.contains(&relocation.instruction_offset) {
            return Err(format!(
                "relocation opcode at 0x{:X} was selected for removal",
                relocation.instruction_offset
            ));
        }
        let target = mapped(&offset_map, relocation.target_offset, "relocation target")?;
        let new_instruction_start = *instruction_new_offsets
            .get(&relocation.instruction_offset)
            .ok_or_else(|| "relocation instruction was not mapped".to_string())?;
        let instruction = instruction_by_offset
            .get(&relocation.instruction_offset)
            .ok_or_else(|| "relocation refers to a missing instruction".to_string())?;
        let new_base = new_instruction_start
            .checked_add(relocation.base_offset - instruction.offset)
            .ok_or_else(|| "relocation base overflows".to_string())?;
        let new_field = new_instruction_start
            .checked_add(relocation.field_offset - instruction.offset)
            .ok_or_else(|| "relocation field overflows".to_string())?;
        let field_end = new_field
            .checked_add(4)
            .ok_or_else(|| "relocation field end overflows".to_string())?;
        if field_end > rebuilt.len() {
            return Err(format!(
                "relocation field 0x{new_field:X} is outside rebuilt payload"
            ));
        }
        if let RelocationKind::JumpTableRelative32 { entry_index } = &relocation.kind {
            let expected = instruction.offset + 4 + usize::from(*entry_index) * 4;
            if instruction.opcode != 2128 || relocation.field_offset != expected {
                return Err("jump table relocation metadata is inconsistent".into());
            }
        }
        let displacement = i64::try_from(target)
            .ok()
            .and_then(|target| {
                i64::try_from(new_base)
                    .ok()
                    .and_then(|base| target.checked_sub(base))
            })
            .ok_or_else(|| "relocation displacement overflows".to_string())?;
        let displacement = i32::try_from(displacement)
            .map_err(|_| format!("relocation displacement does not fit i32 at 0x{new_field:X}"))?;
        rebuilt[new_field..field_end].copy_from_slice(&displacement.to_le_bytes());
    }

    let relocated_symbols = symbols
        .iter()
        .map(|&(hash, offset)| {
            Ok((
                hash,
                u32::try_from(mapped(&offset_map, offset as usize, "symbol offset")?)
                    .map_err(|_| format!("symbol 0x{hash:08X} exceeds u32"))?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let relocated_entrypoint = if entrypoint == u32::MAX {
        u32::MAX
    } else {
        u32::try_from(mapped(&offset_map, entrypoint as usize, "entrypoint")?)
            .map_err(|_| "entrypoint exceeds u32".to_string())?
    };
    Ok(RebuiltPayload {
        payload: rebuilt,
        symbols: relocated_symbols,
        entrypoint: relocated_entrypoint,
        offset_map,
    })
}

fn is_linebreak(instruction: &super::mjo_bytecode::Instruction) -> bool {
    instruction.opcode == 2114 && instruction.text_bytes.as_deref() == Some(b"n\0")
}

fn validate_operand(offset: usize, bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.last() != Some(&0) || bytes[..bytes.len() - 1].contains(&0) {
        return Err(format!(
            "replacement operand at 0x{offset:X} must have one terminal NUL"
        ));
    }
    if bytes.len() > u16::MAX as usize {
        return Err(format!(
            "replacement operand at 0x{offset:X} exceeds u16 length"
        ));
    }
    Ok(())
}

fn insert_boundary(map: &mut BTreeMap<usize, usize>, old: usize, new: usize) -> Result<(), String> {
    if map.insert(old, new).is_some_and(|prior| prior != new) {
        return Err(format!(
            "old boundary 0x{old:X} has conflicting new offsets"
        ));
    }
    Ok(())
}

fn mapped(map: &BTreeMap<usize, usize>, old: usize, what: &str) -> Result<usize, String> {
    map.get(&old)
        .copied()
        .ok_or_else(|| format!("{what} 0x{old:X} is not a decoded boundary"))
}
