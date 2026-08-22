use crate::Result;
use encoding_rs::SHIFT_JIS;
use std::collections::BTreeMap;

pub const HEADER_SIZE: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    pub index_start: usize,
    pub index_len: usize,
    pub bytecode_start: usize,
    pub bytecode_len: usize,
    pub text_start: usize,
    pub text_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRef {
    pub inst_offset: usize,
    pub inst_end: usize,
    pub ref_offset: usize,
    pub target: u16,
    pub opcode: u8,
    pub raw: Vec<u8>,
    pub text: String,
}

#[derive(Debug, Clone, Copy)]
enum Operand {
    Expr,
    String,
}

pub fn read_u16(data: &[u8], offset: usize) -> Result<u16> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or_else(|| format!("u16 at 0x{offset:X} is out of range"))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

pub fn write_u16(data: &mut [u8], offset: usize, value: u16) -> Result<()> {
    let dst = data
        .get_mut(offset..offset + 2)
        .ok_or_else(|| format!("u16 at 0x{offset:X} is out of range"))?;
    dst.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

pub fn parse_layout(data: &[u8], file_name: &str) -> Result<Layout> {
    if data.len() < HEADER_SIZE {
        return Err(format!(
            "{file_name}: file is shorter than the 12-byte AIL header"
        ));
    }
    if data[0..4] != [0, 0, 0, 0] || data[10..12] != [0, 0] {
        return Err(format!("{file_name}: unexpected AIL header constants"));
    }
    let index_len = read_u16(data, 4)? as usize;
    let bytecode_len = read_u16(data, 6)? as usize;
    let text_len = read_u16(data, 8)? as usize;
    if !index_len.is_multiple_of(4) {
        return Err(format!(
            "{file_name}: index byte length is not divisible by 4"
        ));
    }
    let bytecode_start = HEADER_SIZE
        .checked_add(index_len)
        .ok_or_else(|| format!("{file_name}: index length overflow"))?;
    let text_start = bytecode_start
        .checked_add(bytecode_len)
        .ok_or_else(|| format!("{file_name}: bytecode length overflow"))?;
    let expected_len = text_start
        .checked_add(text_len)
        .ok_or_else(|| format!("{file_name}: text length overflow"))?;
    if expected_len != data.len() {
        return Err(format!(
            "{file_name}: declared layout ends at 0x{expected_len:X}, file ends at 0x{:X}",
            data.len()
        ));
    }
    if text_len == 1 {
        return Err(format!("{file_name}: one-byte text pool is invalid"));
    }
    if text_len != 0 && !data.ends_with(&[0, 0]) {
        return Err(format!(
            "{file_name}: text pool is not double-NUL terminated"
        ));
    }
    Ok(Layout {
        index_start: HEADER_SIZE,
        index_len,
        bytecode_start,
        bytecode_len,
        text_start,
        text_len,
    })
}

fn schema(opcode: u8) -> Option<&'static [Operand]> {
    use Operand::{Expr as E, String as S};
    Some(match opcode {
        0x00 | 0x01 | 0x52 | 0xD7 => &[S],
        0x08 | 0x09 | 0x6C | 0x9D | 0xB7 | 0xB8 => &[E, E, E, E, S],
        0x6E | 0xEC => &[S, E],
        0x79 => &[E, E, E, S],
        0x83 | 0x94 => &[E, S],
        0xAA | 0xC9 | 0xCE => &[E, E, S, E, E, E, E, E, E],
        0xD1 => &[E, E, E, S, E, E, E, E, E, E, E, E],
        0xD3 => &[E, E, S, E, E, E, E],
        _ => return None,
    })
}

fn skip_expression(data: &[u8], mut cursor: usize, end: usize) -> Option<usize> {
    let mut stack = 0usize;
    let mut tokens = 0usize;
    while cursor < end && tokens < 512 {
        let token = *data.get(cursor)?;
        cursor += 1;
        tokens += 1;
        match token {
            0x00 => {
                cursor = cursor.checked_add(3)?;
                if cursor > end {
                    return None;
                }
                stack = stack.checked_add(1)?;
            }
            0xFF => return (stack == 1).then_some(cursor),
            _ => {
                let operator = *data.get(cursor)?;
                cursor += 1;
                match operator {
                    10 if stack >= 1 => {}
                    0..=5 | 11 | 12 | 20..=24 if stack >= 2 => stack -= 1,
                    _ => return None,
                }
            }
        }
    }
    None
}

pub fn decode_target(data: &[u8], layout: Layout, target: u16) -> Result<(Vec<u8>, String)> {
    let start = layout
        .text_start
        .checked_add(target as usize)
        .ok_or_else(|| format!("text target 0x{target:04X} overflows"))?;
    let end = layout.text_start + layout.text_len;
    if start >= end {
        return Err(format!(
            "text target 0x{target:04X} is outside the declared pool"
        ));
    }
    let stop = find_double_zero(data, start, end)
        .ok_or_else(|| format!("text target 0x{target:04X} is not terminated"))?;
    let raw = data[start..stop].to_vec();
    let (text, _, had_errors) = SHIFT_JIS.decode(&raw);
    if had_errors {
        return Err(format!("text target 0x{target:04X} is not valid CP932"));
    }
    let text = text.into_owned();
    Ok((raw, text))
}

pub fn find_text_refs_at(
    data: &[u8],
    layout: Layout,
    file_name: &str,
    wanted: &std::collections::BTreeSet<usize>,
) -> Result<Vec<TextRef>> {
    let code_end = layout.bytecode_start + layout.bytecode_len;
    let mut by_ref: BTreeMap<usize, TextRef> = BTreeMap::new();
    for inst_offset in layout.bytecode_start..code_end {
        let opcode = data[inst_offset];
        let Some(operands) = schema(opcode) else {
            continue;
        };
        let mut cursor = inst_offset + 1;
        let mut refs = Vec::new();
        let mut valid = true;
        for operand in operands {
            match operand {
                Operand::Expr => match skip_expression(data, cursor, code_end) {
                    Some(next) => cursor = next,
                    None => {
                        valid = false;
                        break;
                    }
                },
                Operand::String => {
                    if cursor + 2 > code_end {
                        valid = false;
                        break;
                    }
                    let target = read_u16(data, cursor)?;
                    match decode_target(data, layout, target) {
                        Ok((raw, text)) if !raw.is_empty() => refs.push(TextRef {
                            inst_offset,
                            inst_end: 0,
                            ref_offset: cursor,
                            target,
                            opcode,
                            raw,
                            text,
                        }),
                        Ok(_) => {}
                        Err(_) => {
                            valid = false;
                            break;
                        }
                    }
                    cursor += 2;
                }
            }
        }
        if !valid || refs.is_empty() {
            continue;
        }
        for mut text_ref in refs {
            text_ref.inst_end = cursor;
            if !wanted.contains(&text_ref.ref_offset) {
                continue;
            }
            match by_ref.get(&text_ref.ref_offset) {
                None => {
                    by_ref.insert(text_ref.ref_offset, text_ref);
                }
                Some(existing)
                    if existing.inst_offset == text_ref.inst_offset
                        && existing.opcode == text_ref.opcode => {}
                Some(existing) => {
                    return Err(format!(
                        "{file_name}: ambiguous text operand at 0x{:X} (opcodes {:02X}/{:02X})",
                        text_ref.ref_offset, existing.opcode, text_ref.opcode
                    ));
                }
            }
        }
    }
    let missing: Vec<_> = wanted
        .iter()
        .filter(|offset| !by_ref.contains_key(offset))
        .copied()
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "{file_name}: {} reference hint(s) do not belong to a confirmed text opcode; first is 0x{:X}",
            missing.len(), missing[0]
        ));
    }
    Ok(by_ref.into_values().collect())
}

fn find_double_zero(data: &[u8], start: usize, end: usize) -> Option<usize> {
    if start >= end {
        return None;
    }
    (start..end.saturating_sub(1)).find(|&offset| data[offset] == 0 && data[offset + 1] == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expression_grammar_accepts_constant() {
        let bytes = [0x00, 0xFF, 0x34, 0x12, 0xFF];
        assert_eq!(skip_expression(&bytes, 0, bytes.len()), Some(5));
    }

    #[test]
    fn expression_grammar_rejects_unknown_operator() {
        let bytes = [0x00, 0xFF, 1, 0, 0x01, 0x99, 0xFF];
        assert_eq!(skip_expression(&bytes, 0, bytes.len()), None);
    }
}
