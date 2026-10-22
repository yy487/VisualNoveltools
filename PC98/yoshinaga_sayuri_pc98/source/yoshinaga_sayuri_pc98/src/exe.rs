use crate::{decode_text, Result, UiSlot};

const FILE_TO_EA: usize = 0xfc00;
const DATA_FILE_START: usize = 0xd3f0;

fn data_offset(file_offset: usize) -> Result<u16> {
    u16::try_from(
        file_offset
            .checked_sub(DATA_FILE_START)
            .ok_or("EXE string outside data segment")?,
    )
    .map_err(|_| "EXE string exceeds 16-bit data addressing".into())
}
fn patch_word(
    source: &[u8],
    output: &mut [u8],
    ea: usize,
    opcode: &[u8],
    operand: usize,
    value: u16,
) -> Result<()> {
    let offset = ea - FILE_TO_EA;
    if source.get(offset..offset + opcode.len()) != Some(opcode) {
        return Err(format!(
            "EXE code at {ea:05X} differs from the disassembled version"
        ));
    }
    output[offset + operand..offset + operand + 2].copy_from_slice(&value.to_le_bytes());
    Ok(())
}
fn retarget(
    source: &[u8],
    output: &mut [u8],
    slot: &UiSlot,
    destination: usize,
    slots: &[UiSlot],
) -> Result<()> {
    if destination == slot.offset {
        return Ok(());
    }
    let old = data_offset(slot.offset)?;
    let new = data_offset(destination)?;
    for reference in &slot.references {
        let ea = usize::from_str_radix(reference.trim_start_matches("0x"), 16)
            .map_err(|e| e.to_string())?;
        let mut expected = vec![0x68];
        expected.extend(old.to_le_bytes());
        patch_word(source, output, ea, &expected, 1, new)?;
    }
    if slot.category == "save_labels" {
        let index = slots
            .iter()
            .filter(|item| item.category == "save_labels")
            .position(|item| item.offset == slot.offset)
            .ok_or("missing save-label pointer")?;
        let pointer = 0x1d54c - FILE_TO_EA + index * 2;
        if source.get(pointer..pointer + 2) != Some(old.to_le_bytes().as_slice()) {
            return Err("EXE save-label pointer differs from baseline".into());
        }
        output[pointer..pointer + 2].copy_from_slice(&new.to_le_bytes());
    } else if slot.ea == "1DBF8" {
        patch_word(
            source,
            output,
            0x171be,
            &[0xc7, 0x46, 0xf6, 0x08, 0x0c],
            3,
            new,
        )?;
    } else if slot.ea == "1DC19" {
        patch_word(
            source,
            output,
            0x17253,
            &[0xc7, 0x46, 0xf0, 0x29, 0x0c],
            3,
            new,
        )?;
    } else if slot.references.is_empty() {
        return Err(format!(
            "EXE string {} has no verified relocation reference",
            slot.ea
        ));
    }
    Ok(())
}

pub fn rebuild(source: &[u8], slots: &[UiSlot], texts: &[Vec<u8>]) -> Result<Vec<u8>> {
    if slots.len() != texts.len() {
        return Err("EXE text count changed".into());
    }
    let mut output = source.to_vec();
    let mut holes = Vec::new();
    let mut pending = Vec::new();
    let mut destinations = slots.iter().map(|slot| slot.offset).collect::<Vec<_>>();
    for (index, (slot, bytes)) in slots.iter().zip(texts).enumerate() {
        decode_text(bytes, false)
            .map_err(|e| format!("EXE entry {index} ({}): {e}", slot.original))?;
        let end = slot
            .offset
            .checked_add(slot.capacity + 1)
            .ok_or("EXE slot overflow")?;
        if end > source.len() {
            return Err("EXE slot outside file".into());
        }
        if bytes.as_slice() == &source[slot.offset..end - 1] {
            continue;
        }
        output[slot.offset..end].fill(0);
        if bytes.len() <= slot.capacity {
            output[slot.offset..slot.offset + bytes.len()].copy_from_slice(bytes);
            // Only confirmed ordinary C-string tails are reused; dynamic prompts are excluded.
            if slot.category != "disk_messages" && bytes.len() < slot.capacity {
                holes.push((slot.offset + bytes.len() + 1, end));
            }
        } else {
            pending.push(index);
        }
    }
    // Put the longest moved strings first to avoid consuming their only suitable hole.
    pending.sort_by_key(|&index| std::cmp::Reverse(texts[index].len()));
    for index in pending {
        let bytes = &texts[index];
        let needed = bytes.len() + 1;
        let hole = holes.iter_mut().find(|(start, end)| end - *start >= needed).ok_or_else(|| format!("EXE entry {index} ({}): {} encoded bytes; insufficient verified string padding for relocation", slots[index].original, bytes.len()))?;
        let destination = hole.0;
        output[destination..destination + bytes.len()].copy_from_slice(bytes);
        hole.0 += needed;
        destinations[index] = destination;
        retarget(source, &mut output, &slots[index], destination, slots)?;
    }

    let find = |ea: &str| {
        slots
            .iter()
            .position(|slot| slot.ea == ea)
            .ok_or_else(|| format!("missing disk prompt {ea}"))
    };
    let disk = find("1DBF8")?;
    let question = texts[disk]
        .iter()
        .enumerate()
        .filter(|(_, byte)| **byte == b'?')
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if question.len() != 1 {
        return Err(format!(
            "EXE entry {disk}: disk prompt must contain one ASCII '?' placeholder"
        ));
    }
    let target = data_offset(destinations[disk] + question[0])?;
    patch_word(source, &mut output, 0x171c8, &[0xa2, 0x10, 0x0c], 1, target)?;

    let drive = find("1DC04")?;
    let suffix = find("1DC0D")?;
    let particle = find("1DC16")?;
    let prefix_length = texts[drive].len();
    if prefix_length > 127 {
        return Err(format!(
            "EXE entry {drive}: drive prefix exceeds native signed-byte append offset"
        ));
    }
    let offset = 0x171e9 - FILE_TO_EA;
    if source.get(offset..offset + 3) != Some(&[0x83, 0xc7, 8]) {
        return Err("EXE drive append instruction differs from baseline".into());
    }
    output[offset + 2] = prefix_length as u8;
    let overhead = prefix_length
        .checked_add(texts[suffix].len())
        .and_then(|size| size.checked_add(texts[particle].len()))
        .ok_or("EXE prompt allocation overflow")?;
    if overhead > usize::from(u16::MAX) - 52 {
        return Err("EXE composed drive prompt exceeds native allocation range".into());
    }
    patch_word(
        source,
        &mut output,
        0x171d0,
        &[0x05, 0x12, 0],
        1,
        overhead as u16,
    )?;
    for (index, bytes) in texts.iter().enumerate() {
        let start = destinations[index];
        if output.get(start..start + bytes.len()) != Some(bytes.as_slice())
            || output.get(start + bytes.len()) != Some(&0)
        {
            return Err(format!(
                "EXE entry {index}: rebuilt C-string verification failed"
            ));
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{encode_text, UiCatalogue};
    fn fixture() -> (Vec<u8>, Vec<UiSlot>, Vec<Vec<u8>>) {
        let catalogue: UiCatalogue =
            serde_json::from_str(include_str!("ui_catalogue.json")).unwrap();
        let slots = catalogue.entries;
        let texts = slots
            .iter()
            .map(|slot| encode_text(&slot.original, false).unwrap())
            .collect::<Vec<_>>();
        let mut bytes = vec![0; 57736];
        for (slot, text) in slots.iter().zip(&texts) {
            bytes[slot.offset..slot.offset + text.len()].copy_from_slice(text);
        }
        for (ea, code) in [
            (0x171c8, vec![0xa2, 0x10, 0x0c]),
            (0x171d0, vec![0x05, 0x12, 0]),
            (0x171e9, vec![0x83, 0xc7, 8]),
            (0x17239, vec![0x68, 0x1d, 0x0c]),
            (0x1724d, vec![0x68, 0x26, 0x0c]),
        ] {
            bytes[ea - FILE_TO_EA..ea - FILE_TO_EA + code.len()].copy_from_slice(&code);
        }
        (bytes, slots, texts)
    }
    #[test]
    fn shorter_drive_prefix_relocation_and_allocation() {
        let (source, slots, mut texts) = fixture();
        assert_eq!(rebuild(&source, &slots, &texts).unwrap(), source);
        texts[52] = b"A".to_vec();
        texts[55] = b"Disk?".to_vec();
        texts[56] = encode_text("入", false).unwrap();
        texts[57] = encode_text("そのどれかのドライブ", false).unwrap();
        texts[58] = encode_text("にだけ", false).unwrap();
        let result = rebuild(&source, &slots, &texts).unwrap();
        assert_eq!(result[0x75eb], 2);
        assert_eq!(
            u16::from_le_bytes(result[0x75d1..0x75d3].try_into().unwrap()) as usize,
            texts[56].len() + texts[57].len() + texts[58].len()
        );
        let relocated = usize::from(u16::from_le_bytes(
            result[0x763a..0x763c].try_into().unwrap(),
        )) + DATA_FILE_START;
        assert_eq!(&result[relocated..relocated + texts[57].len()], texts[57]);
        assert_eq!(
            u16::from_le_bytes(result[0x75c9..0x75cb].try_into().unwrap()),
            data_offset(slots[55].offset + 4).unwrap()
        );
        assert_eq!(result.len(), source.len());
    }
    #[test]
    fn refuses_unknown_patch_sites_and_missing_placeholder() {
        let (mut source, slots, texts) = fixture();
        source[0x75d0] = 0x90;
        assert!(rebuild(&source, &slots, &texts)
            .unwrap_err()
            .contains("disassembled"));
        let (source, slots, mut texts) = fixture();
        texts[55] = b"Disk".to_vec();
        assert!(rebuild(&source, &slots, &texts)
            .unwrap_err()
            .contains("placeholder"));
    }
}
