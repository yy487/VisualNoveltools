//! Evidence-based decoder for the SOG.EXE text command stream.
use encoding_rs::SHIFT_JIS;
use serde::Serialize;
use std::collections::HashSet;

pub const SCHEMA: &str = "soft-hard-pc98.text-vm.v1";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    Terminator,
    Eof,
    TruncatedInstruction,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordStatus {
    Parsed,
    UnknownDefaultByte,
    Truncated,
}

#[derive(Clone, Debug, Serialize)]
pub struct DecodeDocument {
    pub schema: &'static str,
    pub source: String,
    pub file_size: usize,
    pub parse_start: usize,
    pub parse_limit: usize,
    pub stop_offset: Option<usize>,
    pub stop_reason: StopReason,
    pub prefix_hex: String,
    pub trailing_hex: String,
    pub suffix_hex: String,
    pub instructions: Vec<Instruction>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Instruction {
    pub offset: usize,
    pub end_offset: usize,
    pub opcode: u8,
    pub mnemonic: String,
    pub status: RecordStatus,
    /// Exact bytes from opcode through the last consumed operand/payload terminator.
    pub raw_hex: String,
    pub operands: Vec<Operand>,
    pub strings: Vec<TextPayload>,
    pub branches: Vec<BranchTarget>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Operand {
    pub name: String,
    pub offset: usize,
    pub width: usize,
    pub raw_hex: String,
    pub value: u16,
}

#[derive(Clone, Debug, Serialize)]
pub struct TextPayload {
    pub index: Option<usize>,
    pub offset: usize,
    pub end_offset: usize,
    pub terminator_offset: Option<usize>,
    pub raw_hex: String,
    pub decoded_cp932: String,
    /// Internal source-text view with PC-98 control words represented as markers.
    pub translation: String,
    pub decode_had_errors: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct BranchTarget {
    /// Choice index for opcode 0x0C; absent for ordinary jumps.
    pub choice_index: Option<usize>,
    pub operand_offset: usize,
    /// Relative offsets are based immediately after their own u16 operand.
    pub base_offset: usize,
    /// Signed big-endian relative displacement decoded from the raw u16 bits.
    pub displacement: i16,
    pub target_offset: usize,
    pub conditional: bool,
    pub target_is_instruction_boundary: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
    pub severity: String,
    pub code: String,
    pub offset: usize,
    pub message: String,
}

/// Decode a complete byte slice as one VM stream beginning at offset zero.
pub fn parse(bytes: &[u8], source: impl Into<String>) -> DecodeDocument {
    // A full-file range is always valid.
    parse_range(bytes, source, 0, None).expect("full byte slice is a valid range")
}

/// Decode a bounded VM stream while preserving bytes outside the requested range.
pub fn parse_range(
    bytes: &[u8],
    source: impl Into<String>,
    start: usize,
    length: Option<usize>,
) -> Result<DecodeDocument, String> {
    if start > bytes.len() {
        return Err(format!(
            "起始偏移 {start:#x} 超出文件长度 {:#x}",
            bytes.len()
        ));
    }
    let limit = match length {
        Some(length) => start
            .checked_add(length)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| {
                format!(
                    "解析范围 {start:#x}+{length:#x} 超出文件长度 {:#x}",
                    bytes.len()
                )
            })?,
        None => bytes.len(),
    };

    let mut cursor = start;
    let mut instructions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut stop_offset = None;
    let mut stop_reason = StopReason::Eof;
    let mut trailing_start = limit;

    while cursor < limit {
        let instruction_start = cursor;
        let opcode = bytes[cursor];
        cursor += 1;
        let mut record = Instruction {
            offset: instruction_start,
            end_offset: cursor,
            opcode,
            mnemonic: mnemonic(opcode).to_owned(),
            status: if opcode > 0x12 {
                RecordStatus::UnknownDefaultByte
            } else {
                RecordStatus::Parsed
            },
            raw_hex: String::new(),
            operands: Vec::new(),
            strings: Vec::new(),
            branches: Vec::new(),
        };

        let decoded = decode_instruction(bytes, limit, &mut cursor, &mut record);
        record.end_offset = cursor;
        record.raw_hex = hex(&bytes[instruction_start..cursor]);
        if opcode > 0x12 {
            diagnostics.push(Diagnostic {
                severity: "warning".into(),
                code: "unknown_default_byte".into(),
                offset: instruction_start,
                message: format!(
                    "字节 {opcode:#04x} 进入 EXE 默认分支；目前只确认它按一个字节消费，语义未命名"
                ),
            });
        }
        for payload in &record.strings {
            if payload.decode_had_errors {
                diagnostics.push(Diagnostic {
                    severity: "warning".into(),
                    code: "cp932_decode_replacement".into(),
                    offset: payload.offset,
                    message:
                        "CP932 解码含未定义字节；decoded_cp932 使用替代字符，raw_hex 保留原字节"
                            .into(),
                });
            }
        }

        if let Err(message) = decoded {
            record.status = RecordStatus::Truncated;
            diagnostics.push(Diagnostic {
                severity: "error".into(),
                code: "truncated_instruction".into(),
                offset: instruction_start,
                message,
            });
            record.end_offset = cursor;
            record.raw_hex = hex(&bytes[instruction_start..cursor]);
            instructions.push(record);
            stop_offset = Some(instruction_start);
            stop_reason = StopReason::TruncatedInstruction;
            trailing_start = cursor;
            break;
        }

        let is_terminator = opcode == 0;
        instructions.push(record);
        if is_terminator {
            stop_offset = Some(instruction_start);
            stop_reason = StopReason::Terminator;
            trailing_start = cursor;
            break;
        }
    }

    let boundaries: HashSet<usize> = instructions.iter().map(|item| item.offset).collect();
    for instruction in &mut instructions {
        for branch in &mut instruction.branches {
            branch.target_is_instruction_boundary = boundaries.contains(&branch.target_offset);
            if !branch.target_is_instruction_boundary {
                diagnostics.push(Diagnostic {
                    severity: "warning".into(),
                    code: "branch_target_not_linear_boundary".into(),
                    offset: branch.operand_offset,
                    message: format!(
                        "分支目标 {:#x} 不在本次线性解析得到的指令边界；它可能位于未解析区域或流范围之外",
                        branch.target_offset
                    ),
                });
            }
        }
    }

    if matches!(&stop_reason, StopReason::Eof) {
        stop_offset = Some(limit);
        diagnostics.push(Diagnostic {
            severity: "warning".into(),
            code: "missing_stream_terminator".into(),
            offset: limit,
            message: "在解析范围内到达 EOF，未遇到 opcode 0x00".into(),
        });
    }

    Ok(DecodeDocument {
        schema: SCHEMA,
        source: source.into(),
        file_size: bytes.len(),
        parse_start: start,
        parse_limit: limit,
        stop_offset,
        stop_reason,
        prefix_hex: hex(&bytes[..start]),
        trailing_hex: hex(&bytes[trailing_start..limit]),
        suffix_hex: hex(&bytes[limit..]),
        instructions,
        diagnostics,
    })
}

fn decode_instruction(
    bytes: &[u8],
    limit: usize,
    cursor: &mut usize,
    instruction: &mut Instruction,
) -> Result<(), String> {
    let opcode = instruction.opcode;
    match opcode {
        0x00 | 0x0B | 0x12 => {}
        0x01 => {
            take_u16(bytes, cursor, limit, "object_id", instruction)?;
            take_u16(bytes, cursor, limit, "x", instruction)?;
            take_u16(bytes, cursor, limit, "y", instruction)?;
            take_u8(bytes, cursor, limit, "columns", instruction)?;
            take_u8(bytes, cursor, limit, "rows", instruction)?;
            instruction
                .strings
                .push(take_string(bytes, cursor, limit, None)?);
        }
        0x02 | 0x04 | 0x06 | 0x0A | 0x0D | 0x11 => {
            let name = match opcode {
                0x02 | 0x04 | 0x06 => "object_id",
                0x0A => "msc_id",
                0x0D => "text_object_id",
                _ => "variable_id",
            };
            take_u16(bytes, cursor, limit, name, instruction)?;
            if opcode == 0x0D {
                instruction
                    .strings
                    .push(take_string(bytes, cursor, limit, None)?);
            }
        }
        0x03 | 0x05 => {
            take_u16(bytes, cursor, limit, "image_id", instruction)?;
            take_u16(bytes, cursor, limit, "x", instruction)?;
            take_u16(bytes, cursor, limit, "y", instruction)?;
        }
        0x07 => {
            take_u16(bytes, cursor, limit, "target_variable_id", instruction)?;
            take_u8(bytes, cursor, limit, "arithmetic_code", instruction)?;
            take_u16(bytes, cursor, limit, "immediate", instruction)?;
        }
        0x08 => {
            take_u16(bytes, cursor, limit, "target_variable_id", instruction)?;
            take_u8(bytes, cursor, limit, "arithmetic_code", instruction)?;
            take_u16(bytes, cursor, limit, "source_variable_id", instruction)?;
        }
        0x09 => {
            take_u16(bytes, cursor, limit, "variable_id", instruction)?;
            take_u8(bytes, cursor, limit, "comparison_code", instruction)?;
            take_u16(bytes, cursor, limit, "comparison_value", instruction)?;
            add_branch(bytes, cursor, limit, instruction, None, true)?;
        }
        0x0C => {
            for index in 0..3 {
                instruction
                    .strings
                    .push(take_string(bytes, cursor, limit, Some(index))?);
            }
            for index in 0..3 {
                add_branch(bytes, cursor, limit, instruction, Some(index), true)?;
            }
        }
        0x0E => instruction
            .strings
            .push(take_string(bytes, cursor, limit, None)?),
        0x0F => add_branch(bytes, cursor, limit, instruction, None, false)?,
        0x10 => {
            take_u16(bytes, cursor, limit, "wait_count", instruction)?;
        }
        _ => {
            // Values above 0x12 take the EXE's default path; disassembly confirms
            // the byte has already been consumed before that path is called.
        }
    }
    Ok(())
}

fn take_u16(
    bytes: &[u8],
    cursor: &mut usize,
    limit: usize,
    name: &str,
    instruction: &mut Instruction,
) -> Result<u16, String> {
    let offset = *cursor;
    let available = limit.saturating_sub(offset);
    if available < 2 {
        *cursor = limit;
        return Err(format!(
            "{name} 在 {offset:#x} 需要 2 字节，范围内只剩 {available} 字节"
        ));
    }
    let raw = &bytes[offset..offset + 2];
    let value = u16::from_be_bytes([raw[0], raw[1]]);
    instruction.operands.push(Operand {
        name: name.into(),
        offset,
        width: 2,
        raw_hex: hex(raw),
        value,
    });
    *cursor += 2;
    Ok(value)
}

fn take_u8(
    bytes: &[u8],
    cursor: &mut usize,
    limit: usize,
    name: &str,
    instruction: &mut Instruction,
) -> Result<u8, String> {
    let offset = *cursor;
    if offset >= limit {
        return Err(format!("{name} 在 {offset:#x} 缺少 1 字节操作数"));
    }
    let value = bytes[offset];
    instruction.operands.push(Operand {
        name: name.into(),
        offset,
        width: 1,
        raw_hex: hex(&bytes[offset..offset + 1]),
        value: u16::from(value),
    });
    *cursor += 1;
    Ok(value)
}

fn take_string(
    bytes: &[u8],
    cursor: &mut usize,
    limit: usize,
    index: Option<usize>,
) -> Result<TextPayload, String> {
    let start = *cursor;
    let mut probe = start;
    while probe.saturating_add(1) < limit {
        if bytes[probe] == 0 && bytes[probe + 1] == 0 {
            let raw = &bytes[start..probe];
            let (decoded, _, had_errors) = SHIFT_JIS.decode(raw);
            let payload = TextPayload {
                index,
                offset: start,
                end_offset: probe,
                terminator_offset: Some(probe),
                raw_hex: hex(raw),
                decoded_cp932: decoded.into_owned(),
                translation: String::new(),
                decode_had_errors: had_errors,
            };
            *cursor = probe + 2;
            return Ok(payload);
        }
        probe += 2;
    }
    let remaining = limit.saturating_sub(start);
    *cursor = limit;
    Err(format!(
        "NUL-u16 文本串从 {start:#x} 开始，范围内没有完整的 00 00 终止符（剩余 {remaining} 字节）"
    ))
}

fn add_branch(
    bytes: &[u8],
    cursor: &mut usize,
    limit: usize,
    instruction: &mut Instruction,
    choice_index: Option<usize>,
    conditional: bool,
) -> Result<(), String> {
    let operand_offset = *cursor;
    let raw_displacement = take_u16(bytes, cursor, limit, "relative_displacement", instruction)?;
    let displacement = i16::from_be_bytes(raw_displacement.to_be_bytes());
    let base_offset = *cursor;
    let target_offset = if displacement >= 0 {
        base_offset.checked_add(displacement as usize)
    } else {
        base_offset.checked_sub(usize::from(displacement.unsigned_abs()))
    }
    .ok_or_else(|| format!("分支目标偏移在 {operand_offset:#x} 超出脚本范围"))?;
    instruction.branches.push(BranchTarget {
        choice_index,
        operand_offset,
        base_offset,
        displacement,
        target_offset,
        conditional,
        target_is_instruction_boundary: false,
    });
    Ok(())
}

fn mnemonic(opcode: u8) -> &'static str {
    match opcode {
        0x00 => "end_stream",
        0x01 => "define_text_grid",
        0x02 => "remove_object_with_wait",
        0x03 => "show_sh_image",
        0x04 => "remove_object_bank_500",
        0x05 => "show_shc_image",
        0x06 => "remove_object_bank_1000",
        0x07 => "modify_variable_literal",
        0x08 => "modify_variable_from_variable",
        0x09 => "branch_if",
        0x0A => "load_msc_resource",
        0x0B => "wait_subsystem",
        0x0C => "three_choice",
        0x0D => "select_text_object_and_print",
        0x0E => "print_text",
        0x0F => "jump_relative",
        0x10 => "wait_ticks",
        0x11 => "print_variable",
        0x12 => "cast_sequence",
        _ => "default_unknown_byte",
    }
}

pub fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 3);
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        use std::fmt::Write as _;
        let _ = write!(out, "{byte:02X}");
    }
    out
}
