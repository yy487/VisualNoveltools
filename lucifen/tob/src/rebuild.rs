use crate::format::{parse, read_u32, CodeRegion, ParameterKind, TobFile};
use crate::translation::Replacement;
use crate::Result;
use std::collections::BTreeMap;
use std::ops::Range;

#[derive(Debug, Clone)]
struct Edit {
    range: Range<usize>,
    bytes: Vec<u8>,
}

pub fn rebuild(file: &TobFile, replacements: Vec<Replacement>) -> Result<Vec<u8>> {
    let mut edits = Vec::new();
    let mut command_deltas = BTreeMap::<usize, isize>::new();
    for replacement in replacements {
        let original = file
            .bytes
            .get(replacement.slot.range.clone())
            .ok_or("replacement slot exceeds the source TOB")?;
        if original == replacement.bytes {
            continue;
        }
        let delta = isize::try_from(replacement.bytes.len())
            .and_then(|new| isize::try_from(original.len()).map(|old| new - old))
            .map_err(|_| "replacement length exceeds isize")?;
        edits.push(Edit {
            range: replacement.slot.range.clone(),
            bytes: replacement.bytes.clone(),
        });
        if let Some(length_pos) = replacement.slot.length_pos {
            let terminated_length = replacement
                .bytes
                .len()
                .checked_add(1)
                .ok_or("string length overflow")?;
            let encoded = u16::try_from(terminated_length)
                .map_err(|_| "inline string exceeds the TOB u16 length field")?;
            edits.push(Edit {
                range: length_pos..length_pos + 2,
                bytes: encoded.to_le_bytes().to_vec(),
            });
        }
        if let Some(command_start) = replacement.slot.command_start {
            *command_deltas.entry(command_start).or_default() += delta;
        }
    }
    if edits.is_empty() {
        return Ok(file.bytes.clone());
    }

    for (command_start, delta) in command_deltas {
        let command = file
            .regions
            .iter()
            .find_map(|region| match region {
                CodeRegion::Command(command) if command.start == command_start => Some(command),
                _ => None,
            })
            .ok_or_else(|| format!("replacement references unknown command 0x{command_start:x}"))?;
        let old_size = command.end - command.size_pos;
        let new_size = old_size
            .checked_add_signed(delta)
            .ok_or("command size underflow/overflow")?;
        let encoded = u32::try_from(new_size).map_err(|_| "command exceeds its u32 size field")?;
        edits.push(Edit {
            range: command.size_pos..command.size_pos + 4,
            bytes: encoded.to_le_bytes().to_vec(),
        });
    }
    edits.sort_by_key(|edit| edit.range.start);
    for pair in edits.windows(2) {
        if pair[0].range.end > pair[1].range.start {
            return Err(format!(
                "overlapping rebuild edits at 0x{:x} and 0x{:x}",
                pair[0].range.start, pair[1].range.start
            ));
        }
    }

    let mut rebuilt = apply_edits(&file.bytes, &edits)?;
    let mapped_code_start = map_point(file.code_start, &edits)?;
    if mapped_code_start != file.code_start {
        return Err("TOB header size changed unexpectedly".to_string());
    }

    for label in &file.labels {
        rewrite_target(
            &mut rebuilt,
            label.target_pos,
            label.target,
            file.code_start,
            &edits,
        )?;
    }
    for (position, target) in file.offset_target_positions.iter().zip(&file.offsets) {
        rewrite_target(&mut rebuilt, *position, *target, file.code_start, &edits)?;
    }
    for command in file.regions.iter().filter_map(|region| match region {
        CodeRegion::Command(command) if command.opcode <= 3 => Some(command),
        _ => None,
    }) {
        let first = command
            .parameters
            .first()
            .ok_or("validated jump lost its first parameter")?;
        if !matches!(first.kind, ParameterKind::Immediate | ParameterKind::Dword) {
            return Err("validated jump target is no longer static".to_string());
        }
        let value_pos = first
            .value_pos
            .ok_or("static jump target has no position")?;
        let target = read_u32(&file.bytes, value_pos, "jump target")?;
        rewrite_target(&mut rebuilt, value_pos, target, file.code_start, &edits)?;
    }

    parse(rebuilt.clone()).map_err(|error| format!("rebuilt TOB validation failed: {error}"))?;
    Ok(rebuilt)
}

fn rewrite_target(
    rebuilt: &mut [u8],
    old_value_pos: usize,
    old_target: u32,
    code_start: usize,
    edits: &[Edit],
) -> Result<()> {
    if old_target == u32::MAX {
        return Ok(());
    }
    let old_absolute = code_start
        .checked_add(old_target as usize)
        .ok_or("TOB target absolute offset overflow")?;
    let new_absolute = map_point(old_absolute, edits)?;
    let new_target = new_absolute
        .checked_sub(code_start)
        .ok_or("relocated TOB target precedes the code")?;
    let encoded = u32::try_from(new_target).map_err(|_| "relocated TOB target exceeds u32")?;
    let new_value_pos = map_point(old_value_pos, edits)?;
    let destination = rebuilt
        .get_mut(new_value_pos..new_value_pos + 4)
        .ok_or("relocated target field exceeds rebuilt TOB")?;
    destination.copy_from_slice(&encoded.to_le_bytes());
    Ok(())
}

fn apply_edits(source: &[u8], edits: &[Edit]) -> Result<Vec<u8>> {
    let total_delta = edits.iter().try_fold(0isize, |total, edit| {
        let old = isize::try_from(edit.range.len()).map_err(|_| "edit length exceeds isize")?;
        let new = isize::try_from(edit.bytes.len()).map_err(|_| "edit length exceeds isize")?;
        total.checked_add(new - old).ok_or("rebuilt size overflow")
    })?;
    let capacity = source
        .len()
        .checked_add_signed(total_delta)
        .ok_or("rebuilt size underflow/overflow")?;
    let mut output = Vec::with_capacity(capacity);
    let mut cursor = 0usize;
    for edit in edits {
        if edit.range.start < cursor || edit.range.end > source.len() {
            return Err("invalid or overlapping edit range".to_string());
        }
        output.extend_from_slice(&source[cursor..edit.range.start]);
        output.extend_from_slice(&edit.bytes);
        cursor = edit.range.end;
    }
    output.extend_from_slice(&source[cursor..]);
    Ok(output)
}

fn map_point(old: usize, edits: &[Edit]) -> Result<usize> {
    let mut delta = 0isize;
    for edit in edits {
        if old < edit.range.start {
            break;
        }
        if old == edit.range.start {
            return old
                .checked_add_signed(delta)
                .ok_or_else(|| "mapped offset overflow".to_string());
        }
        if old < edit.range.end {
            return Err(format!(
                "structural reference points inside edited bytes at 0x{old:x}"
            ));
        }
        let old_length = isize::try_from(edit.range.len()).map_err(|_| "edit length overflow")?;
        let new_length = isize::try_from(edit.bytes.len()).map_err(|_| "edit length overflow")?;
        delta = delta
            .checked_add(new_length - old_length)
            .ok_or("mapped offset delta overflow")?;
    }
    old.checked_add_signed(delta)
        .ok_or_else(|| "mapped offset overflow".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_boundaries_around_a_growing_edit() {
        let edits = vec![Edit {
            range: 10..12,
            bytes: vec![1, 2, 3, 4],
        }];
        assert_eq!(map_point(10, &edits).unwrap(), 10);
        assert_eq!(map_point(12, &edits).unwrap(), 14);
        assert!(map_point(11, &edits).is_err());
    }
}
