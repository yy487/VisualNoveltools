//! Static instruction-boundary parser for MajiroObj V1 bytecode.
//!
//! The opcode set and operand widths below follow the VM dispatch in
//! `カタハネ ―An' call Belle―.exe_export_for_ai/decompile/439450.c`. This parser
//! does not execute the VM: it decodes each symbol-delimited function, records
//! relocations, and rejects unknown opcodes, truncated operands, and targets
//! that do not land on a decoded instruction boundary.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

const ENTRYPOINT_SENTINEL: u32 = u32::MAX;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BytecodeProgram {
    pub(super) functions: Vec<FunctionRange>,
    pub(super) instructions: Vec<Instruction>,
    pub(super) relocations: Vec<RelocationField>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FunctionRange {
    /// Payload-relative start, shared by every symbol hash in `symbol_hashes`.
    pub(super) start: usize,
    /// Exclusive payload-relative end, at the next function start or payload end.
    pub(super) end: usize,
    pub(super) symbol_hashes: Vec<u32>,
    /// Half-open indices into `BytecodeProgram::instructions`.
    pub(super) first_instruction: usize,
    pub(super) instruction_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Instruction {
    /// Payload-relative opcode offset.
    pub(super) offset: usize,
    pub(super) opcode: u16,
    /// Opcode and all its operands, in bytes.
    pub(super) size: usize,
    /// Original length-prefixed bytes for opcodes 2049, 2112, and 2114.
    pub(super) text_bytes: Option<Vec<u8>>,
    /// Function hash for script-call opcodes 2063/2064.
    pub(super) script_call_hash: Option<u32>,
    /// Argument count for script-call opcodes 2063/2064.
    pub(super) script_call_argument_count: Option<u16>,
    /// Opcode 2113 marks a display event. It has no static operand; runtime may
    /// rewind the PC and replay this opcode, which does not change its width.
    pub(super) display_boundary: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RelocationField {
    /// Payload-relative offset of the signed little-endian rel32 field.
    pub(super) field_offset: usize,
    /// Payload-relative PC base: the byte immediately after this field.
    pub(super) base_offset: usize,
    /// Resolved payload-relative target, retained for rebuilding after edits.
    pub(super) target_offset: usize,
    pub(super) instruction_offset: usize,
    pub(super) kind: RelocationKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum RelocationKind {
    /// Direct conditional/unconditional branch operand.
    BranchRelative32,
    /// Relative target stored for later execution by VM state.
    DeferredBranchRelative32,
    /// One independently based entry in opcode 2128's counted switch table.
    JumpTableRelative32 { entry_index: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperandSpec {
    None,
    Fixed(usize),
    LengthPrefixedBytes,
    CountedBytes,
    JumpTable,
    Relative32(RelocationKindTag),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RelocationKindTag {
    Branch,
    DeferredBranch,
}

/// Decode all functions from a V1-decoded MJO payload.
///
/// `symbols` contains `(hash, payload-relative entry offset)` pairs. Input
/// order is not assumed to be address order; equal offsets are grouped while
/// preserving their associated hashes.
pub(super) fn parse_payload(
    payload: &[u8],
    symbols: &[(u32, u32)],
    entrypoint: u32,
) -> Result<BytecodeProgram, String> {
    if payload.len() > u32::MAX as usize {
        return Err("MJO 负载超出 32 位地址范围".into());
    }

    let mut starts: BTreeMap<usize, Vec<u32>> = BTreeMap::new();
    for &(hash, raw_offset) in symbols {
        let start = raw_offset as usize;
        if start >= payload.len() {
            return Err(format!(
                "符号 0x{hash:08X} 的函数偏移 0x{start:X} 不在非空负载范围内"
            ));
        }
        starts.entry(start).or_default().push(hash);
    }

    if entrypoint != ENTRYPOINT_SENTINEL && !starts.contains_key(&(entrypoint as usize)) {
        return Err(format!(
            "MJO 入口偏移 0x{entrypoint:X} 不是符号函数起点，也不是无入口哨兵"
        ));
    }
    if starts.is_empty() {
        if payload.is_empty() {
            return Ok(BytecodeProgram {
                functions: Vec::new(),
                instructions: Vec::new(),
                relocations: Vec::new(),
            });
        }
        return Err("MJO 负载非空，但符号表没有函数起点，无法安全确定指令范围".into());
    }

    let mut sorted_starts = starts.into_iter().collect::<Vec<_>>();
    for (_, hashes) in &mut sorted_starts {
        hashes.sort_unstable();
    }

    let mut program = BytecodeProgram {
        functions: Vec::with_capacity(sorted_starts.len()),
        instructions: Vec::new(),
        relocations: Vec::new(),
    };

    for index in 0..sorted_starts.len() {
        let (start, symbol_hashes) = &sorted_starts[index];
        let end = sorted_starts
            .get(index + 1)
            .map_or(payload.len(), |(next_start, _)| *next_start);
        if *start >= end {
            return Err(format!("函数范围无效: [0x{start:X}, 0x{end:X})"));
        }

        let first_instruction = program.instructions.len();
        let mut cursor = *start;
        while cursor < end {
            let instruction_offset = cursor;
            let opcode = read_u16(payload, cursor, end, instruction_offset, "opcode")?;
            cursor = checked_advance(cursor, 2, end, instruction_offset, "opcode")?;
            let spec = operand_spec(opcode).ok_or_else(|| {
                format!(
                    "未知 MJO opcode 0x{opcode:04X}，负载偏移 0x{instruction_offset:X}（函数 0x{start:X}）"
                )
            })?;

            let mut text_bytes = None;
            let mut script_call_hash = None;
            let mut script_call_argument_count = None;
            match spec {
                OperandSpec::None => {}
                OperandSpec::Fixed(length) => {
                    if matches!(opcode, 2063 | 2064) {
                        let hash_bytes = checked_slice(
                            payload,
                            cursor,
                            4,
                            end,
                            instruction_offset,
                            "脚本调用函数哈希",
                        )?;
                        script_call_hash = Some(u32::from_le_bytes(
                            hash_bytes.try_into().expect("four-byte function hash"),
                        ));
                        let argument_count = checked_slice(
                            payload,
                            cursor + 8,
                            2,
                            end,
                            instruction_offset,
                            "脚本调用参数数量",
                        )?;
                        script_call_argument_count =
                            Some(u16::from_le_bytes([argument_count[0], argument_count[1]]));
                    }
                    cursor =
                        checked_advance(cursor, length, end, instruction_offset, "固定操作数")?;
                }
                OperandSpec::LengthPrefixedBytes => {
                    let length =
                        read_u16(payload, cursor, end, instruction_offset, "字符串长度")? as usize;
                    cursor = checked_advance(cursor, 2, end, instruction_offset, "字符串长度")?;
                    let bytes = checked_slice(
                        payload,
                        cursor,
                        length,
                        end,
                        instruction_offset,
                        "长度前缀字符串",
                    )?;
                    text_bytes = Some(bytes.to_vec());
                    cursor += length;
                }
                OperandSpec::CountedBytes => {
                    let count =
                        read_u16(payload, cursor, end, instruction_offset, "字节计数")? as usize;
                    cursor = checked_advance(cursor, 2, end, instruction_offset, "字节计数")?;
                    checked_slice(
                        payload,
                        cursor,
                        count,
                        end,
                        instruction_offset,
                        "计数字节数组",
                    )?;
                    cursor += count;
                }
                OperandSpec::JumpTable => {
                    let count = read_u16(payload, cursor, end, instruction_offset, "跳转表项数")?;
                    cursor = checked_advance(cursor, 2, end, instruction_offset, "跳转表项数")?;
                    if count == 0 {
                        return Err(format!(
                            "opcode 2128 的跳转表为空，负载偏移 0x{instruction_offset:X}"
                        ));
                    }
                    let table_bytes = usize::from(count)
                        .checked_mul(4)
                        .ok_or_else(|| format!("跳转表长度溢出，偏移 0x{instruction_offset:X}"))?;
                    checked_slice(
                        payload,
                        cursor,
                        table_bytes,
                        end,
                        instruction_offset,
                        "跳转表",
                    )?;
                    for entry_index in 0..count {
                        let field_offset = cursor + usize::from(entry_index) * 4;
                        let base_offset = field_offset + 4;
                        let displacement = read_i32(
                            payload,
                            field_offset,
                            end,
                            instruction_offset,
                            "跳转表 rel32",
                        )?;
                        let target_offset = relative_target(
                            base_offset,
                            displacement,
                            payload.len(),
                            instruction_offset,
                        )?;
                        program.relocations.push(RelocationField {
                            field_offset,
                            base_offset,
                            target_offset,
                            instruction_offset,
                            kind: RelocationKind::JumpTableRelative32 { entry_index },
                        });
                    }
                    cursor += table_bytes;
                }
                OperandSpec::Relative32(kind) => {
                    let field_offset = cursor;
                    let displacement = read_i32(
                        payload,
                        field_offset,
                        end,
                        instruction_offset,
                        "rel32 分支位移",
                    )?;
                    cursor = checked_advance(cursor, 4, end, instruction_offset, "rel32 分支位移")?;
                    let base_offset = cursor;
                    let target_offset = relative_target(
                        base_offset,
                        displacement,
                        payload.len(),
                        instruction_offset,
                    )?;
                    let kind = match kind {
                        RelocationKindTag::Branch => RelocationKind::BranchRelative32,
                        RelocationKindTag::DeferredBranch => {
                            RelocationKind::DeferredBranchRelative32
                        }
                    };
                    program.relocations.push(RelocationField {
                        field_offset,
                        base_offset,
                        target_offset,
                        instruction_offset,
                        kind,
                    });
                }
            }

            let size = cursor - instruction_offset;
            if size < 2 {
                return Err(format!(
                    "opcode 长度无效，负载偏移 0x{instruction_offset:X}"
                ));
            }
            program.instructions.push(Instruction {
                offset: instruction_offset,
                opcode,
                size,
                text_bytes,
                script_call_hash,
                script_call_argument_count,
                display_boundary: opcode == 2113,
            });
        }
        if cursor != end {
            return Err(format!(
                "函数 0x{start:X} 的解析未精确落到下一函数边界 0x{end:X}"
            ));
        }
        program.functions.push(FunctionRange {
            start: *start,
            end,
            symbol_hashes: symbol_hashes.clone(),
            first_instruction,
            instruction_end: program.instructions.len(),
        });
    }

    let mut boundaries = program
        .instructions
        .iter()
        .map(|instruction| instruction.offset)
        .collect::<BTreeSet<_>>();
    boundaries.insert(payload.len());
    for relocation in &program.relocations {
        if !boundaries.contains(&relocation.target_offset) {
            return Err(format!(
                "rel32 目标 0x{:X} 不是指令边界或负载终点（opcode 位于 0x{:X}）",
                relocation.target_offset, relocation.instruction_offset
            ));
        }
    }

    Ok(program)
}

/// Return instruction indices reachable from any symbol function entry.
///
/// Opcode 2091 returns from the current script frame, while opcode 2092 is an
/// unconditional rel32 branch in the VM dispatcher. Other relative branches
/// and switch tables keep both their target edges and fallthrough edges here;
/// that conservative over-approximation avoids dropping text behind conditional
/// or deferred control flow.
pub(super) fn reachable_instruction_indices(program: &BytecodeProgram) -> BTreeSet<usize> {
    let instruction_indices = program
        .instructions
        .iter()
        .enumerate()
        .map(|(index, instruction)| (instruction.offset, index))
        .collect::<BTreeMap<_, _>>();
    let mut targets_by_instruction = BTreeMap::<usize, Vec<usize>>::new();
    for relocation in &program.relocations {
        targets_by_instruction
            .entry(relocation.instruction_offset)
            .or_default()
            .push(relocation.target_offset);
    }

    let mut reachable = BTreeSet::new();
    let mut function_end_by_instruction = vec![0usize; program.instructions.len()];
    let mut pending = VecDeque::new();
    for function in &program.functions {
        if function.first_instruction >= function.instruction_end {
            continue;
        }
        for end in
            &mut function_end_by_instruction[function.first_instruction..function.instruction_end]
        {
            *end = function.instruction_end;
        }
        pending.push_back(function.first_instruction);
    }

    while let Some(index) = pending.pop_front() {
        if index >= program.instructions.len()
            || function_end_by_instruction[index] == 0
            || !reachable.insert(index)
        {
            continue;
        }
        let instruction = &program.instructions[index];
        if instruction.opcode == 2091 {
            continue;
        }

        if let Some(targets) = targets_by_instruction.get(&instruction.offset) {
            for target in targets {
                if let Some(target_index) = instruction_indices.get(target) {
                    pending.push_back(*target_index);
                }
            }
        }

        if instruction.opcode != 2092 && index + 1 < function_end_by_instruction[index] {
            pending.push_back(index + 1);
        }
    }
    reachable
}

fn operand_spec(opcode: u16) -> Option<OperandSpec> {
    use OperandSpec::{
        CountedBytes, Fixed, JumpTable, LengthPrefixedBytes, None as NoOperands, Relative32,
    };
    use RelocationKindTag::{Branch, DeferredBranch};

    Some(match opcode {
        // Numeric constants and address-like local variable operands.
        2048 | 2051 => Fixed(4),
        2050 | 2103 => Fixed(8),

        // Strings / counted inline byte arrays.
        2049 | 2112 | 2114 => LengthPrefixedBytes,
        2089 | 2102 => CountedBytes,

        // Script calls: hash + runtime cache slot + argc.
        2063 | 2064 => Fixed(10),
        // Native calls: hash + argc.
        2100 | 2101 => Fixed(6),

        // Direct PC-relative branches (field base is immediately after rel32).
        2092 | 2093 | 2094 | 2096..=2099 | 2104 | 2105 => Relative32(Branch),
        // PC-relative destinations stored in VM continuation state.
        2107..=2109 | 2115 | 2117 | 2119 => Relative32(DeferredBranch),
        2128 => JumpTable,
        2106 => Fixed(2),

        // Variable access / assignment families. Each handler reads u16 + u32,
        // then advances over a trailing u16, for eight operand bytes total.
        432
        | 528
        | 433
        | 529
        | 434..=437
        | 530..=533
        | 440
        | 536
        | 441
        | 537
        | 448
        | 544
        | 449
        | 545
        | 456
        | 552
        | 464
        | 560
        | 465
        | 561
        | 466
        | 562
        | 472
        | 568
        | 473
        | 569
        | 480
        | 576
        | 488
        | 584
        | 496
        | 592
        | 504
        | 600
        | 512
        | 608
        | 624
        | 720
        | 625
        | 721
        | 626
        | 722
        | 632
        | 728
        | 633
        | 729
        | 640
        | 736
        | 641
        | 737
        | 648
        | 744
        | 656
        | 752
        | 657
        | 753
        | 658
        | 754
        | 664
        | 760
        | 665
        | 761
        | 672
        | 768
        | 680
        | 776
        | 688
        | 784
        | 696
        | 792
        | 704
        | 800 => Fixed(8),

        // The comparison handlers 314 and 330 read only VM stack values in
        // 439450.c; they do not consume inline PC operands.
        // All remaining listed opcodes explicitly dispatched by 439450.c have
        // no inline operand. Keeping this list explicit makes unknown opcodes
        // fail closed instead of being mistaken for one-word instructions.
        256
        | 257
        | 264
        | 265
        | 272
        | 280..=282
        | 288
        | 289
        | 296
        | 304
        | 312
        | 313
        | 314
        | 320..=322
        | 328
        | 329
        | 330
        | 336..=338
        | 344..=349
        | 352..=357
        | 360
        | 368
        | 376
        | 384
        | 392
        | 400
        | 401
        | 408
        | 416
        | 417
        | 424
        | 425
        | 2091
        | 2095
        | 2110
        | 2111
        | 2113
        | 2116
        | 2118 => NoOperands,
        _ => return None,
    })
}

fn checked_advance(
    cursor: usize,
    amount: usize,
    function_end: usize,
    instruction_offset: usize,
    what: &str,
) -> Result<usize, String> {
    let end = cursor
        .checked_add(amount)
        .ok_or_else(|| format!("{what}偏移溢出，指令位于 0x{instruction_offset:X}"))?;
    if end > function_end {
        return Err(format!(
            "{what}越过函数边界，指令位于 0x{instruction_offset:X}，需要到 0x{end:X}，函数终点为 0x{function_end:X}"
        ));
    }
    Ok(end)
}

fn checked_slice<'a>(
    payload: &'a [u8],
    offset: usize,
    length: usize,
    function_end: usize,
    instruction_offset: usize,
    what: &str,
) -> Result<&'a [u8], String> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| format!("{what}长度溢出，指令位于 0x{instruction_offset:X}"))?;
    if end > function_end {
        return Err(format!(
            "{what}越过函数边界，指令位于 0x{instruction_offset:X}，结束偏移 0x{end:X}，函数终点 0x{function_end:X}"
        ));
    }
    payload
        .get(offset..end)
        .ok_or_else(|| format!("{what}超出负载，指令位于 0x{instruction_offset:X}"))
}

fn read_u16(
    payload: &[u8],
    offset: usize,
    function_end: usize,
    instruction_offset: usize,
    what: &str,
) -> Result<u16, String> {
    let bytes = checked_slice(payload, offset, 2, function_end, instruction_offset, what)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_i32(
    payload: &[u8],
    offset: usize,
    function_end: usize,
    instruction_offset: usize,
    what: &str,
) -> Result<i32, String> {
    let bytes = checked_slice(payload, offset, 4, function_end, instruction_offset, what)?;
    Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn relative_target(
    base_offset: usize,
    displacement: i32,
    payload_length: usize,
    instruction_offset: usize,
) -> Result<usize, String> {
    let target = (base_offset as i64) + i64::from(displacement);
    if target < 0 || target > payload_length as i64 {
        return Err(format!(
            "rel32 目标越出负载: 指令 0x{instruction_offset:X}，基址 0x{base_offset:X}，位移 {displacement}"
        ));
    }
    Ok(target as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(bytes: &mut Vec<u8>, opcode: u16) {
        bytes.extend_from_slice(&opcode.to_le_bytes());
    }

    fn symbol(hash: u32, offset: usize) -> (u32, u32) {
        (hash, offset as u32)
    }

    #[test]
    fn parses_text_operands_and_2113_boundary() {
        let mut payload = Vec::new();
        op(&mut payload, 2049);
        payload.extend_from_slice(&3u16.to_le_bytes());
        payload.extend_from_slice(b"A\0B");
        op(&mut payload, 2112);
        payload.extend_from_slice(&2u16.to_le_bytes());
        payload.extend_from_slice(b"x\0");
        op(&mut payload, 2113);
        op(&mut payload, 2114);
        payload.extend_from_slice(&4u16.to_le_bytes());
        payload.extend_from_slice(b"d%1\0");

        let parsed = parse_payload(&payload, &[symbol(0x1234, 0)], 0).unwrap();
        assert_eq!(parsed.instructions.len(), 4);
        assert_eq!(parsed.instructions[0].size, 7);
        assert_eq!(
            parsed.instructions[0].text_bytes.as_deref(),
            Some(&b"A\0B"[..])
        );
        assert_eq!(
            parsed.instructions[1].text_bytes.as_deref(),
            Some(&b"x\0"[..])
        );
        assert!(parsed.instructions[2].display_boundary);
        assert_eq!(
            parsed.instructions[3].text_bytes.as_deref(),
            Some(&b"d%1\0"[..])
        );
    }

    #[test]
    fn skips_counted_byte_vectors_without_scanning_their_contents() {
        let mut payload = Vec::new();
        op(&mut payload, 2089);
        payload.extend_from_slice(&3u16.to_le_bytes());
        payload.extend_from_slice(&[0x00, 0x08, 0xff]); // Bytes resemble opcode material.
        op(&mut payload, 2102);
        payload.extend_from_slice(&2u16.to_le_bytes());
        payload.extend_from_slice(&[0x3b, 0x08]);
        op(&mut payload, 2091);

        let parsed = parse_payload(&payload, &[symbol(1, 0)], 0).unwrap();
        assert_eq!(parsed.instructions.len(), 3);
        assert_eq!(parsed.instructions[0].offset, 0);
        assert_eq!(parsed.instructions[0].size, 7);
        assert_eq!(parsed.instructions[1].offset, 7);
        assert_eq!(parsed.instructions[1].size, 6);
        assert_eq!(parsed.instructions[2].offset, 13);
    }

    #[test]
    fn groups_alias_symbol_offsets_and_preserves_hashes() {
        let mut payload = Vec::new();
        op(&mut payload, 2091);
        op(&mut payload, 2091);
        let parsed = parse_payload(
            &payload,
            &[symbol(0xB, 2), symbol(0xA, 0), symbol(0xC, 0)],
            2,
        )
        .unwrap();
        assert_eq!(parsed.functions.len(), 2);
        assert_eq!(parsed.functions[0].start, 0);
        assert_eq!(parsed.functions[0].symbol_hashes, vec![0xA, 0xC]);
        assert_eq!(parsed.functions[1].start, 2);
        assert_eq!(parsed.functions[1].symbol_hashes, vec![0xB]);
    }

    #[test]
    fn resolves_branch_from_end_of_rel32_field() {
        let mut payload = Vec::new();
        op(&mut payload, 2048);
        payload.extend_from_slice(&7i32.to_le_bytes());
        op(&mut payload, 2092);
        payload.extend_from_slice(&0i32.to_le_bytes());
        op(&mut payload, 2113);
        op(&mut payload, 2091);

        let parsed = parse_payload(&payload, &[symbol(1, 0)], 0).unwrap();
        assert_eq!(
            parsed
                .instructions
                .iter()
                .map(|instruction| instruction.offset)
                .collect::<Vec<_>>(),
            vec![0, 6, 12, 14]
        );
        assert_eq!(parsed.relocations.len(), 1);
        assert_eq!(parsed.relocations[0].field_offset, 8);
        assert_eq!(parsed.relocations[0].base_offset, 12);
        assert_eq!(parsed.relocations[0].target_offset, 12);
        assert_eq!(parsed.relocations[0].kind, RelocationKind::BranchRelative32);
    }

    #[test]
    fn records_each_jump_table_entry_with_its_own_base() {
        let mut payload = Vec::new();
        op(&mut payload, 2128);
        payload.extend_from_slice(&2u16.to_le_bytes());
        payload.extend_from_slice(&8i32.to_le_bytes()); // field end 8 -> offset 16
        payload.extend_from_slice(&4i32.to_le_bytes()); // field end 12 -> offset 16
        op(&mut payload, 2113);
        op(&mut payload, 2113);
        op(&mut payload, 2091);

        let parsed = parse_payload(&payload, &[symbol(1, 0)], 0).unwrap();
        assert_eq!(parsed.instructions[0].size, 12);
        assert_eq!(parsed.relocations.len(), 2);
        assert_eq!(parsed.relocations[0].field_offset, 4);
        assert_eq!(parsed.relocations[0].base_offset, 8);
        assert_eq!(parsed.relocations[0].target_offset, 16);
        assert_eq!(
            parsed.relocations[0].kind,
            RelocationKind::JumpTableRelative32 { entry_index: 0 }
        );
        assert_eq!(parsed.relocations[1].field_offset, 8);
        assert_eq!(parsed.relocations[1].base_offset, 12);
        assert_eq!(parsed.relocations[1].target_offset, 16);
    }

    #[test]
    fn rejects_unknown_truncated_and_bad_targets() {
        let unknown = 0x7fffu16.to_le_bytes();
        assert!(parse_payload(&unknown, &[symbol(1, 0)], 0)
            .unwrap_err()
            .contains("未知 MJO opcode"));

        let truncated = [0x01, 0x08, 0x04, 0x00, b'a']; // 2049 claims four bytes.
        assert!(parse_payload(&truncated, &[symbol(1, 0)], 0).is_err());

        let mut bad_target = Vec::new();
        op(&mut bad_target, 2092);
        bad_target.extend_from_slice(&1i32.to_le_bytes()); // target offset 7, inside opcode.
        op(&mut bad_target, 2091);
        assert!(parse_payload(&bad_target, &[symbol(1, 0)], 0)
            .unwrap_err()
            .contains("不是指令边界"));
    }

    #[test]
    fn rejects_unrecognized_entrypoint_and_non_instruction_tail() {
        let payload = [0x3b, 0x08]; // 2107 with a truncated rel32.
        assert!(parse_payload(&payload, &[symbol(1, 0)], 0).is_err());

        let payload = [0x3b, 0x08, 0, 0, 0, 0, 0]; // One stray byte after a complete opcode.
        assert!(parse_payload(&payload, &[symbol(1, 0)], 0).is_err());

        let payload = [0x3b, 0x08, 0, 0, 0, 0]; // A rel32 target at function terminal is valid.
        assert!(parse_payload(&payload, &[symbol(1, 0)], 0).is_ok());
        assert!(parse_payload(&payload, &[symbol(1, 0)], 1)
            .unwrap_err()
            .contains("不是符号函数起点"));
    }
}
