//! Length-changing reconstruction for statically parsed MajiroObj V1 payloads.

use std::collections::BTreeMap;

use super::mjo_bytecode::{parse_payload, BytecodeProgram, RelocationKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RebuiltPayload {
    /// Rebuilt V1-decoded payload bytes.
    pub(super) payload: Vec<u8>,
    /// New `(hash, payload-relative offset)` pairs in the original table order.
    pub(super) symbols: Vec<(u32, u32)>,
    /// New payload-relative entry point, or the original `u32::MAX` sentinel.
    pub(super) entrypoint: u32,
    /// Mapping for every old instruction start/end boundary and payload end.
    pub(super) offset_map: BTreeMap<usize, usize>,
}

/// Rebuild a parsed payload after replacing length-prefixed text operands.
///
/// Replacement keys are payload-relative instruction offsets. Replacement
/// values are complete byte strings, including exactly one trailing NUL.
pub(super) fn rebuild_payload(
    payload: &[u8],
    program: &BytecodeProgram,
    replacements: &BTreeMap<usize, Vec<u8>>,
    symbols: &[(u32, u32)],
    entrypoint: u32,
) -> Result<RebuiltPayload, String> {
    if payload.len() > u32::MAX as usize {
        return Err("原始 MJO 负载超出 32 位地址范围".into());
    }

    // Reparse against the caller's source metadata. Besides checking offsets,
    // this confirms the supplied instruction and relocation lists are complete.
    let verified = parse_payload(payload, symbols, entrypoint)
        .map_err(|error| format!("重建输入无法重新解析：{error}"))?;
    if &verified != program {
        return Err("重建输入的字节码解析结果与当前负载、符号表或入口点不一致".into());
    }

    let text_instructions = program
        .instructions
        .iter()
        .filter(|instruction| matches!(instruction.opcode, 2049 | 2112 | 2114))
        .map(|instruction| (instruction.offset, instruction))
        .collect::<BTreeMap<_, _>>();

    for (&offset, replacement) in replacements {
        let instruction = text_instructions
            .get(&offset)
            .ok_or_else(|| format!("替换键 0x{offset:X} 不是已识别的文本指令起点"))?;
        if replacement.is_empty() || replacement.last() != Some(&0) {
            return Err(format!("文本指令 0x{offset:X} 的替换内容必须包含末尾 NUL"));
        }
        if replacement[..replacement.len() - 1].contains(&0) {
            return Err(format!(
                "文本指令 0x{offset:X} 的替换内容包含内部 NUL，无法作为单个脚本字符串注入"
            ));
        }
        if replacement.len() > u16::MAX as usize {
            return Err(format!(
                "文本指令 0x{offset:X} 的替换长度 {} 超出 u16 范围",
                replacement.len()
            ));
        }
        if instruction.text_bytes.is_none() {
            return Err(format!("文本指令 0x{offset:X} 缺少解析后的原始字符串"));
        }
    }

    let mut rebuilt = Vec::with_capacity(payload.len());
    let mut offset_map = BTreeMap::new();
    let mut instruction_new_offsets = BTreeMap::new();

    for instruction in &program.instructions {
        let old_start = instruction.offset;
        let old_end = old_start
            .checked_add(instruction.size)
            .ok_or_else(|| format!("指令 0x{old_start:X} 的结束位置溢出"))?;
        let new_start = rebuilt.len();
        insert_boundary(&mut offset_map, old_start, new_start)?;
        instruction_new_offsets.insert(old_start, new_start);

        let source = payload
            .get(old_start..old_end)
            .ok_or_else(|| format!("指令 0x{old_start:X} 超出原始负载"))?;
        if matches!(instruction.opcode, 2049 | 2112 | 2114) {
            let original_text = instruction
                .text_bytes
                .as_deref()
                .ok_or_else(|| format!("文本指令 0x{old_start:X} 缺少字符串操作数"))?;
            let text = replacements
                .get(&old_start)
                .map_or(original_text, Vec::as_slice);
            let encoded_length = u16::try_from(text.len())
                .map_err(|_| format!("文本指令 0x{old_start:X} 长度超出 u16 范围"))?;

            // The parser verified this source instruction's opcode and length
            // field, so retain its opcode bytes and regenerate only the length
            // and string operand.
            rebuilt.extend_from_slice(&source[..2]);
            rebuilt.extend_from_slice(&encoded_length.to_le_bytes());
            rebuilt.extend_from_slice(text);
        } else {
            // All non-text bytes remain byte-for-byte identical here. The only
            // later edits are the known rel32 relocation fields.
            rebuilt.extend_from_slice(source);
        }

        insert_boundary(&mut offset_map, old_end, rebuilt.len())?;
    }

    match program.instructions.last() {
        Some(last) if last.offset.checked_add(last.size) == Some(payload.len()) => {}
        None if payload.is_empty() => {}
        _ => return Err("解析指令没有完整覆盖原始负载".into()),
    }
    insert_boundary(&mut offset_map, payload.len(), rebuilt.len())?;
    if rebuilt.len() > u32::MAX as usize {
        return Err("重建后的 MJO 负载超出 32 位地址范围".into());
    }

    // Every stored code address must refer to a mapped instruction boundary (or
    // the payload end), as guaranteed by the parser. Do not interpolate through
    // edited string contents: they are data, not executable address targets.
    for relocation in &program.relocations {
        let new_target = mapped_boundary(&offset_map, relocation.target_offset, "rel32 目标")?;
        let instruction_new_start = *instruction_new_offsets
            .get(&relocation.instruction_offset)
            .ok_or_else(|| {
                format!(
                    "rel32 指令偏移 0x{:X} 不在解析指令表中",
                    relocation.instruction_offset
                )
            })?;
        let instruction = program
            .instructions
            .iter()
            .find(|instruction| instruction.offset == relocation.instruction_offset)
            .ok_or_else(|| "rel32 重定位引用了不存在的指令".to_string())?;
        let instruction_end = instruction
            .offset
            .checked_add(instruction.size)
            .ok_or_else(|| "rel32 指令范围溢出".to_string())?;
        let old_field_end = relocation
            .field_offset
            .checked_add(4)
            .ok_or_else(|| "rel32 原字段末尾溢出".to_string())?;
        if relocation.field_offset < instruction.offset || old_field_end > instruction_end {
            return Err(format!(
                "rel32 字段 0x{:X} 不完整地位于指令 0x{:X} 内",
                relocation.field_offset, instruction.offset
            ));
        }
        if relocation.base_offset < instruction.offset || relocation.base_offset > instruction_end {
            return Err(format!(
                "rel32 基址 0x{:X} 不在指令 0x{:X} 的范围内",
                relocation.base_offset, instruction.offset
            ));
        }
        let base_intra_offset = relocation.base_offset - instruction.offset;
        let new_base = instruction_new_start
            .checked_add(base_intra_offset)
            .ok_or_else(|| "rel32 新基址溢出".to_string())?;
        let intra_instruction_offset = relocation
            .field_offset
            .checked_sub(relocation.instruction_offset)
            .ok_or_else(|| "rel32 字段位于其指令起点之前".to_string())?;
        let new_field = instruction_new_start
            .checked_add(intra_instruction_offset)
            .ok_or_else(|| "重定位字段新偏移溢出".to_string())?;
        let field_end = new_field
            .checked_add(4)
            .ok_or_else(|| "重定位字段末尾溢出".to_string())?;
        if field_end > rebuilt.len() {
            return Err(format!("rel32 字段的新偏移 0x{new_field:X} 超出重建负载"));
        }

        if let RelocationKind::JumpTableRelative32 { entry_index } = &relocation.kind {
            let expected_field = instruction
                .offset
                .checked_add(4)
                .and_then(|base| base.checked_add(usize::from(*entry_index).checked_mul(4)?))
                .ok_or_else(|| "跳转表重定位字段偏移溢出".to_string())?;
            if instruction.opcode != 2128 || relocation.field_offset != expected_field {
                return Err(format!(
                    "opcode 2128 的第 {entry_index} 项重定位元数据不一致"
                ));
            }
        }

        let displacement = i64::try_from(new_target)
            .ok()
            .and_then(|target| {
                i64::try_from(new_base)
                    .ok()
                    .and_then(|base| target.checked_sub(base))
            })
            .ok_or_else(|| "rel32 新位移计算溢出".to_string())?;
        let displacement = i32::try_from(displacement).map_err(|_| {
            format!(
                "rel32 新位移超出 i32 范围：指令 0x{:X}，目标 0x{new_target:X}，基址 0x{new_base:X}",
                relocation.instruction_offset
            )
        })?;
        rebuilt[new_field..field_end].copy_from_slice(&displacement.to_le_bytes());
    }

    let mut rebuilt_symbols = Vec::with_capacity(symbols.len());
    for &(hash, old_offset) in symbols {
        let new_offset = mapped_boundary(&offset_map, old_offset as usize, "符号偏移")?;
        rebuilt_symbols.push((
            hash,
            u32::try_from(new_offset)
                .map_err(|_| format!("符号 0x{hash:08X} 的新偏移超出 u32 范围"))?,
        ));
    }

    let rebuilt_entrypoint = if entrypoint == u32::MAX {
        u32::MAX
    } else {
        let new_offset = mapped_boundary(&offset_map, entrypoint as usize, "入口点")?;
        u32::try_from(new_offset).map_err(|_| "新入口点超出 u32 范围".to_string())?
    };

    Ok(RebuiltPayload {
        payload: rebuilt,
        symbols: rebuilt_symbols,
        entrypoint: rebuilt_entrypoint,
        offset_map,
    })
}

fn insert_boundary(
    offsets: &mut BTreeMap<usize, usize>,
    old_offset: usize,
    new_offset: usize,
) -> Result<(), String> {
    if let Some(previous) = offsets.insert(old_offset, new_offset) {
        if previous != new_offset {
            return Err(format!(
                "旧负载边界 0x{old_offset:X} 对应多个新偏移（0x{previous:X} 和 0x{new_offset:X}）"
            ));
        }
    }
    Ok(())
}

fn mapped_boundary(
    offsets: &BTreeMap<usize, usize>,
    old_offset: usize,
    what: &str,
) -> Result<usize, String> {
    offsets
        .get(&old_offset)
        .copied()
        .ok_or_else(|| format!("{what} 0x{old_offset:X} 不是可重建的指令边界"))
}
