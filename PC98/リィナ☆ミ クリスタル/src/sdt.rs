use std::{collections::BTreeSet, path::Path};

use anyhow::{Context, Result, bail, ensure};

use crate::{
    codec::{
        CharacterMap, CharacterSubstitutions, decode_choice, decode_mixed, encode_choice,
        encode_mixed,
    },
    model::{EntryKind, TranslationEntry, TranslationFile},
};

const SCHEMA: &str = "liena-sdt-v1";

#[derive(Debug)]
pub struct ExtractedSdt {
    pub translation: TranslationFile,
    bytes: Vec<u8>,
    entries: Vec<ParsedEntry>,
    target_fields: Vec<TargetField>,
}

#[derive(Debug)]
pub struct InjectedSdt {
    pub bytes: Vec<u8>,
    pub changed_entries: usize,
    pub substituted_characters: usize,
}

#[derive(Debug)]
struct ParsedEntry {
    payload_start: usize,
    payload_end: usize,
    name_wrapper: Option<NameWrapper>,
}

#[derive(Debug)]
struct NameWrapper {
    before_name: String,
    between_name_and_message: String,
}

#[derive(Debug)]
struct TargetField {
    position: usize,
    target: usize,
    owner: usize,
    opcode: u8,
    follows_return: bool,
}

#[derive(Debug)]
struct Splice {
    start: usize,
    end: usize,
    replacement: Vec<u8>,
    entry_index: usize,
}

pub fn parse_sdt(source_file: &Path, bytes: Vec<u8>, map: &CharacterMap) -> Result<ExtractedSdt> {
    ensure!(
        !source_file.is_absolute(),
        "source file must be relative, got {}",
        source_file.display()
    );
    ensure!(
        bytes.len() <= u16::MAX as usize,
        "SDT is {} bytes, exceeding the 16-bit address space",
        bytes.len()
    );

    let file = source_file.to_string_lossy().replace('\\', "/");
    let mut translation_entries = Vec::new();
    let mut parsed_entries = Vec::new();
    let mut target_fields = Vec::new();
    let mut instruction_starts = BTreeSet::new();
    let mut cursor = 0;
    let mut previous_opcode = None;

    while cursor < bytes.len() {
        let instruction_offset = cursor;
        let opcode = bytes[cursor];
        instruction_starts.insert(instruction_offset);

        match opcode {
            0x10 | 0x18 => {
                let payload_start = cursor + 1;
                let payload_end = find_nul(&bytes, payload_start, instruction_offset, opcode)?;
                let decoded =
                    decode_mixed(&bytes[payload_start..payload_end], map).with_context(|| {
                        format!(
                            "invalid mixed text at 0x{payload_start:04X} (opcode 0x{opcode:02X})"
                        )
                    })?;
                let split = split_name(&decoded);
                let index = translation_entries.len();
                let name = split.name;
                translation_entries.push(TranslationEntry {
                    file: file.clone(),
                    index,
                    offset: instruction_offset,
                    size: payload_end + 1 - instruction_offset,
                    kind: EntryKind::Message,
                    opcode: format!("0x{opcode:02X}"),
                    name: name.clone(),
                    scr_name: name,
                    scr_msg: split.message.clone(),
                    message: split.message,
                });
                parsed_entries.push(ParsedEntry {
                    payload_start,
                    payload_end,
                    name_wrapper: split.wrapper,
                });
                cursor = payload_end + 1;
            }
            0x11 | 0x1B | 0x22 | 0xF0 => {
                cursor += 1;
            }
            0x12 | 0x13 | 0x82 | 0x84 | 0xA0 | 0xA1 | 0xB2 | 0xBC | 0xBD | 0xBF | 0xFA => {
                cursor = checked_end(&bytes, cursor, 2, opcode)?;
            }
            0x20 | 0x21 => {
                cursor = checked_end(&bytes, cursor, 3, opcode)?;
                target_fields.push(TargetField {
                    position: instruction_offset + 1,
                    target: read_u16(&bytes, instruction_offset + 1),
                    owner: instruction_offset,
                    opcode,
                    follows_return: previous_opcode == Some(0x22),
                });
            }
            0x30 => {
                cursor = parse_choice_block(
                    &file,
                    &bytes,
                    instruction_offset,
                    map,
                    &mut translation_entries,
                    &mut parsed_entries,
                )?;
            }
            0x40 | 0x41 | 0x42 | 0x45 | 0x85 | 0xE1 | 0xE2 => {
                cursor = checked_end(&bytes, cursor, 3, opcode)?;
            }
            0x50 | 0x51 => {
                let target_position =
                    parse_condition(&bytes, cursor + 1, instruction_offset, opcode)?;
                cursor = checked_end(&bytes, target_position, 2, opcode)?;
                target_fields.push(TargetField {
                    position: target_position,
                    target: read_u16(&bytes, target_position),
                    owner: instruction_offset,
                    opcode,
                    follows_return: false,
                });
            }
            0x52 => {
                let string_start = parse_condition(&bytes, cursor + 1, instruction_offset, opcode)?;
                cursor = find_nul(&bytes, string_start, instruction_offset, opcode)? + 1;
            }
            0x53 => {
                let arguments = parse_condition(&bytes, cursor + 1, instruction_offset, opcode)?;
                cursor = checked_end(&bytes, arguments, 2, opcode)?;
            }
            0x54 => {
                cursor = parse_condition(&bytes, cursor + 1, instruction_offset, opcode)?;
            }
            0x80 | 0xB0 => {
                let string_start = checked_end(&bytes, cursor, 2, opcode)?;
                cursor = find_nul(&bytes, string_start, instruction_offset, opcode)? + 1;
            }
            0x81 | 0xB1 | 0xBA | 0xBB | 0xE0 => {
                let string_start = cursor + 1;
                cursor = find_nul(&bytes, string_start, instruction_offset, opcode)? + 1;
            }
            0x87 => {
                let subtype_end = checked_end(&bytes, cursor, 2, opcode)?;
                cursor = if bytes[cursor + 1] <= 2 {
                    checked_end(&bytes, cursor, 3, opcode)?
                } else {
                    subtype_end
                };
            }
            0x8A => {
                let string_start = checked_end(&bytes, cursor, 3, opcode)?;
                cursor = find_nul(&bytes, string_start, instruction_offset, opcode)? + 1;
            }
            0x8B => {
                let first_start = checked_end(&bytes, cursor, 3, opcode)?;
                let second_start = find_nul(&bytes, first_start, instruction_offset, opcode)? + 1;
                cursor = find_nul(&bytes, second_start, instruction_offset, opcode)? + 1;
            }
            _ => bail!("unknown opcode 0x{opcode:02X} at 0x{instruction_offset:04X}"),
        }

        previous_opcode = Some(opcode);
    }

    target_fields.retain(|field| !is_known_dead_jump(field, &bytes));
    for field in &target_fields {
        ensure!(
            instruction_starts.contains(&field.target),
            "opcode 0x{:02X} at 0x{:04X} targets 0x{:04X}, which is not an instruction boundary",
            field.opcode,
            field.owner,
            field.target
        );
    }

    Ok(ExtractedSdt {
        translation: TranslationFile {
            schema: SCHEMA.to_owned(),
            source_file: file,
            entries: translation_entries,
        },
        bytes,
        entries: parsed_entries,
        target_fields,
    })
}

impl ExtractedSdt {
    pub fn inject(self, translation: &TranslationFile, map: &CharacterMap) -> Result<InjectedSdt> {
        self.inject_with_substitutions(translation, map, &CharacterSubstitutions::default())
    }

    pub fn inject_with_substitutions(
        self,
        translation: &TranslationFile,
        map: &CharacterMap,
        substitutions: &CharacterSubstitutions,
    ) -> Result<InjectedSdt> {
        ensure!(
            translation.schema == SCHEMA,
            "unsupported schema {:?}",
            translation.schema
        );
        ensure!(
            translation
                .source_file
                .eq_ignore_ascii_case(&self.translation.source_file),
            "translation source_file {:?} does not match {:?}",
            translation.source_file,
            self.translation.source_file
        );
        ensure!(
            translation.entries.len() == self.translation.entries.len(),
            "translation entry count {} does not match source count {}",
            translation.entries.len(),
            self.translation.entries.len()
        );

        let (normalized_translation, substituted_characters) =
            apply_substitutions(&self.translation, translation, substitutions);
        let translation = &normalized_translation;

        let mut splices = Vec::new();
        for ((source, translated), parsed) in self
            .translation
            .entries
            .iter()
            .zip(&translation.entries)
            .zip(&self.entries)
        {
            validate_entry_metadata(source, translated)?;
            let changed = translated.name != source.name || translated.message != source.message;
            if !changed {
                continue;
            }

            let replacement = match source.kind {
                EntryKind::Message => encode_message(parsed, source, translated, map)?,
                EntryKind::Choice => {
                    ensure!(
                        translated.name.is_none(),
                        "choice entry {} cannot have a name",
                        source.index
                    );
                    encode_choice(&translated.message, map)
                        .with_context(|| format!("invalid choice entry {}", source.index))?
                }
            };
            splices.push(Splice {
                start: parsed.payload_start,
                end: parsed.payload_end,
                replacement,
                entry_index: source.index,
            });
        }

        ensure_non_overlapping(&splices)?;
        let new_len = rebuilt_len(self.bytes.len(), &splices)?;
        ensure!(
            new_len <= u16::MAX as usize,
            "rebuilt SDT is {new_len} bytes, exceeding the 16-bit address space"
        );

        let mut rebuilt = self.bytes.clone();
        for splice in splices.iter().rev() {
            rebuilt.splice(splice.start..splice.end, splice.replacement.iter().copied());
        }

        for field in &self.target_fields {
            let position = remap_offset(field.position, &splices, "target field")?;
            let target = remap_offset(field.target, &splices, "control-flow target")?;
            ensure!(
                target <= u16::MAX as usize,
                "remapped target 0x{target:X} is out of range"
            );
            let target = (target as u16).to_le_bytes();
            let slot = rebuilt.get_mut(position..position + 2).with_context(|| {
                format!("remapped target field at 0x{position:04X} is out of range")
            })?;
            slot.copy_from_slice(&target);
        }

        let reparsed = parse_sdt(
            Path::new(&self.translation.source_file),
            rebuilt.clone(),
            map,
        )
        .context("rebuilt SDT failed structural validation")?;
        ensure_reparsed_matches(&reparsed.translation, translation)?;

        Ok(InjectedSdt {
            bytes: rebuilt,
            changed_entries: splices.len(),
            substituted_characters,
        })
    }
}

fn apply_substitutions(
    source: &TranslationFile,
    translation: &TranslationFile,
    substitutions: &CharacterSubstitutions,
) -> (TranslationFile, usize) {
    let mut normalized = translation.clone();
    let mut count = 0;
    for ((source_entry, translated_entry), normalized_entry) in source
        .entries
        .iter()
        .zip(&translation.entries)
        .zip(&mut normalized.entries)
    {
        if translated_entry.name != source_entry.name
            && let Some(name) = &translated_entry.name
        {
            let (converted, converted_count) = substitutions.apply(name);
            normalized_entry.name = Some(converted);
            count += converted_count;
        }
        if translated_entry.message != source_entry.message {
            let (converted, converted_count) = substitutions.apply(&translated_entry.message);
            normalized_entry.message = converted;
            count += converted_count;
        }
    }
    (normalized, count)
}

fn parse_choice_block(
    file: &str,
    bytes: &[u8],
    instruction_offset: usize,
    map: &CharacterMap,
    translation_entries: &mut Vec<TranslationEntry>,
    parsed_entries: &mut Vec<ParsedEntry>,
) -> Result<usize> {
    let mut cursor = checked_end(bytes, instruction_offset, 3, 0x30)?;
    loop {
        let marker = *bytes
            .get(cursor)
            .with_context(|| format!("unterminated choice block at 0x{instruction_offset:04X}"))?;
        if marker == 0xFF {
            return Ok(cursor + 1);
        }

        cursor = parse_condition(bytes, cursor, instruction_offset, 0x30)?;
        cursor = checked_end(bytes, cursor, 1, 0x30)?; // choice id
        let payload_start = cursor;
        let payload_end = find_nul(bytes, payload_start, instruction_offset, 0x30)?;
        let payload = &bytes[payload_start..payload_end];
        ensure!(
            payload.len() <= 40,
            "choice label at 0x{payload_start:04X} is {} bytes; runtime limit is 40",
            payload.len()
        );
        let message = decode_choice(payload, map)
            .with_context(|| format!("invalid choice label at 0x{payload_start:04X}"))?;
        let index = translation_entries.len();
        translation_entries.push(TranslationEntry {
            file: file.to_owned(),
            index,
            offset: payload_start,
            size: payload_end + 1 - payload_start,
            kind: EntryKind::Choice,
            opcode: "0x30".to_owned(),
            name: None,
            scr_name: None,
            scr_msg: message.clone(),
            message,
        });
        parsed_entries.push(ParsedEntry {
            payload_start,
            payload_end,
            name_wrapper: None,
        });
        cursor = payload_end + 1;
    }
}

fn parse_condition(
    bytes: &[u8],
    mut cursor: usize,
    instruction_offset: usize,
    opcode: u8,
) -> Result<usize> {
    loop {
        let end = checked_end(bytes, cursor, 4, opcode).with_context(|| {
            format!("truncated condition for instruction at 0x{instruction_offset:04X}")
        })?;
        match bytes[cursor + 3] {
            0 => return Ok(end),
            2 => cursor = end,
            join => {
                bail!(
                    "invalid condition join 0x{join:02X} at 0x{:04X} for instruction at 0x{instruction_offset:04X}",
                    cursor + 3
                )
            }
        }
    }
}

fn validate_entry_metadata(source: &TranslationEntry, translated: &TranslationEntry) -> Result<()> {
    ensure!(
        source.index == translated.index,
        "entry index mismatch at {}",
        source.index
    );
    ensure!(
        source.file == translated.file,
        "_file mismatch at entry {}",
        source.index
    );
    ensure!(
        source.offset == translated.offset,
        "_offset mismatch at entry {}",
        source.index
    );
    ensure!(
        source.size == translated.size,
        "_size mismatch at entry {}",
        source.index
    );
    ensure!(
        source.kind == translated.kind,
        "_type mismatch at entry {}",
        source.index
    );
    ensure!(
        source.opcode == translated.opcode,
        "_opcode mismatch at entry {}",
        source.index
    );
    ensure!(
        source.scr_msg == translated.scr_msg,
        "scr_msg mismatch at entry {}",
        source.index
    );
    ensure!(
        source.scr_name == translated.scr_name,
        "_scr_name mismatch at entry {}",
        source.index
    );
    match source.name {
        Some(_) => ensure!(
            translated.name.is_some(),
            "named message entry {} must keep a name field",
            source.index
        ),
        None => ensure!(
            translated.name.is_none(),
            "unnamed entry {} cannot add a name",
            source.index
        ),
    }
    Ok(())
}

fn encode_message(
    parsed: &ParsedEntry,
    source: &TranslationEntry,
    translated: &TranslationEntry,
    map: &CharacterMap,
) -> Result<Vec<u8>> {
    let mut text = String::new();
    if let Some(wrapper) = &parsed.name_wrapper {
        let name = translated
            .name
            .as_deref()
            .with_context(|| format!("named message entry {} is missing name", source.index))?;
        ensure!(
            !name.contains(['\0', '\r', '\n', '<', '>', '【', '】']),
            "name at entry {} contains a control, newline, NUL, or structural bracket",
            source.index
        );
        text.push_str(&wrapper.before_name);
        text.push_str(name);
        text.push_str(&wrapper.between_name_and_message);
    } else {
        ensure!(
            translated.name.is_none(),
            "unnamed message entry {} cannot add a name",
            source.index
        );
    }
    text.push_str(&translated.message);
    encode_mixed(&text, map).with_context(|| format!("invalid message entry {}", source.index))
}

fn ensure_non_overlapping(splices: &[Splice]) -> Result<()> {
    for pair in splices.windows(2) {
        ensure!(
            pair[0].end <= pair[1].start,
            "entry {} overlaps entry {}",
            pair[0].entry_index,
            pair[1].entry_index
        );
    }
    Ok(())
}

fn rebuilt_len(original_len: usize, splices: &[Splice]) -> Result<usize> {
    let mut length = original_len;
    for splice in splices {
        length = length
            .checked_sub(splice.end - splice.start)
            .and_then(|value| value.checked_add(splice.replacement.len()))
            .context("rebuilt SDT length overflow")?;
    }
    Ok(length)
}

fn remap_offset(offset: usize, splices: &[Splice], role: &str) -> Result<usize> {
    let mut delta = 0_isize;
    for splice in splices {
        if offset < splice.start {
            break;
        }
        if offset == splice.start {
            return offset
                .checked_add_signed(delta)
                .context("remapped offset overflow");
        }
        if offset < splice.end {
            if splice.replacement.len() != splice.end - splice.start {
                bail!(
                    "{role} 0x{offset:04X} lies inside resized entry {} payload 0x{:04X}..0x{:04X}",
                    splice.entry_index,
                    splice.start,
                    splice.end
                );
            }
            return (splice.start + offset - splice.start)
                .checked_add_signed(delta)
                .context("remapped offset overflow");
        }
        delta += splice.replacement.len() as isize - (splice.end - splice.start) as isize;
    }
    offset
        .checked_add_signed(delta)
        .context("remapped offset overflow")
}

fn ensure_reparsed_matches(actual: &TranslationFile, expected: &TranslationFile) -> Result<()> {
    ensure!(
        actual.entries.len() == expected.entries.len(),
        "rebuilt extraction has {} entries, expected {}",
        actual.entries.len(),
        expected.entries.len()
    );
    for (actual, expected) in actual.entries.iter().zip(&expected.entries) {
        ensure!(
            actual.index == expected.index,
            "rebuilt entry order changed"
        );
        ensure!(
            actual.kind == expected.kind,
            "rebuilt entry {} changed type",
            actual.index
        );
        ensure!(
            actual.name == expected.name,
            "rebuilt entry {} name does not match requested translation",
            actual.index
        );
        ensure!(
            actual.scr_msg == expected.message,
            "rebuilt entry {} message does not match requested translation",
            actual.index
        );
    }
    Ok(())
}

struct SplitText {
    name: Option<String>,
    message: String,
    wrapper: Option<NameWrapper>,
}

fn split_name(decoded: &str) -> SplitText {
    const STATE: &str = "<$>";
    const OPEN: char = '【';
    const CLOSE: char = '】';

    let Some(after_first) = decoded.strip_prefix(STATE) else {
        return SplitText {
            name: None,
            message: decoded.to_owned(),
            wrapper: None,
        };
    };
    let Some(second_relative) = after_first.find(STATE) else {
        return SplitText {
            name: None,
            message: decoded.to_owned(),
            wrapper: None,
        };
    };
    let second = STATE.len() + second_relative;
    let candidate = &decoded[STATE.len()..second];
    let Some(open_relative) = candidate.find(OPEN) else {
        return no_name(decoded);
    };
    let Some(close_relative) = candidate.rfind(CLOSE) else {
        return no_name(decoded);
    };
    let close_end = close_relative + CLOSE.len_utf8();
    if open_relative >= close_relative
        || close_end != candidate.len()
        || !is_control_sequence(&candidate[..open_relative])
    {
        return no_name(decoded);
    }
    let name_start = STATE.len() + open_relative + OPEN.len_utf8();
    let name_end = STATE.len() + close_relative;
    let name = &decoded[name_start..name_end];
    if name.contains([OPEN, CLOSE]) {
        return no_name(decoded);
    }

    SplitText {
        name: Some(name.to_owned()),
        message: decoded[second + STATE.len()..].to_owned(),
        wrapper: Some(NameWrapper {
            before_name: decoded[..name_start].to_owned(),
            between_name_and_message: decoded[name_end..second + STATE.len()].to_owned(),
        }),
    }
}

fn no_name(decoded: &str) -> SplitText {
    SplitText {
        name: None,
        message: decoded.to_owned(),
        wrapper: None,
    }
}

fn is_control_sequence(mut text: &str) -> bool {
    while !text.is_empty() {
        let Some(rest) = text.strip_prefix('<') else {
            return false;
        };
        let Some(end) = rest.find('>') else {
            return false;
        };
        text = &rest[end + 1..];
    }
    true
}

fn checked_end(bytes: &[u8], start: usize, length: usize, opcode: u8) -> Result<usize> {
    let end = start
        .checked_add(length)
        .context("instruction offset overflow")?;
    ensure!(
        end <= bytes.len(),
        "truncated opcode 0x{opcode:02X} at 0x{start:04X}: need {length} bytes"
    );
    Ok(end)
}

fn find_nul(bytes: &[u8], start: usize, instruction_offset: usize, opcode: u8) -> Result<usize> {
    bytes
        .get(start..)
        .and_then(|tail| tail.iter().position(|byte| *byte == 0))
        .map(|relative| start + relative)
        .with_context(|| {
            format!("unterminated string for opcode 0x{opcode:02X} at 0x{instruction_offset:04X}")
        })
}

fn read_u16(bytes: &[u8], position: usize) -> usize {
    u16::from_le_bytes([bytes[position], bytes[position + 1]]) as usize
}

fn is_known_dead_jump(field: &TargetField, bytes: &[u8]) -> bool {
    field.opcode == 0x20
        && field.follows_return
        && bytes.get(field.owner..field.owner + 3) == Some([0x20, 0x20, 0x1F].as_slice())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_sdt() -> Vec<u8> {
        let map = CharacterMap::default();
        let text = encode_mixed("<$><c6>【リィナ】<$>本文<g0>", &map).unwrap();
        let choice = encode_choice("選択", &map).unwrap();
        let choice_offset = 3 + 1 + text.len() + 1;

        let mut bytes = vec![0x20, choice_offset as u8, (choice_offset >> 8) as u8, 0x10];
        bytes.extend(text);
        bytes.push(0);
        bytes.extend([0x30, 0x00, 0x00]);
        bytes.extend([0x01, 0x02, 0x03, 0x00]);
        bytes.push(0x07);
        bytes.extend(choice);
        bytes.extend([0x00, 0xFF, 0xF0]);
        bytes
    }

    #[test]
    fn parses_named_message_and_choice() {
        let parsed = parse_sdt(
            Path::new("LC01A.SDT"),
            sample_sdt(),
            &CharacterMap::default(),
        )
        .unwrap();
        assert_eq!(parsed.translation.entries.len(), 2);
        assert_eq!(
            parsed.translation.entries[0].name.as_deref(),
            Some("リィナ")
        );
        assert_eq!(parsed.translation.entries[0].message, "本文<g0>");
        assert_eq!(parsed.translation.entries[1].kind, EntryKind::Choice);
        assert_eq!(parsed.translation.entries[1].message, "選択");
    }

    #[test]
    fn unchanged_injection_is_byte_exact() {
        let source = sample_sdt();
        let parsed = parse_sdt(
            Path::new("LC01A.SDT"),
            source.clone(),
            &CharacterMap::default(),
        )
        .unwrap();
        let translation = parsed.translation.clone();
        let rebuilt = parsed
            .inject(&translation, &CharacterMap::default())
            .unwrap();
        assert_eq!(rebuilt.changed_entries, 0);
        assert_eq!(rebuilt.substituted_characters, 0);
        assert_eq!(rebuilt.bytes, source);
    }

    #[test]
    fn injection_applies_substitutions_only_to_edited_fields() {
        let source = sample_sdt();
        let parsed = parse_sdt(Path::new("LC01A.SDT"), source, &CharacterMap::default()).unwrap();
        let mut translation = parsed.translation.clone();
        translation.entries[0].name = Some("你".to_owned());
        translation.entries[0].message = "你说".to_owned();
        let substitutions = CharacterSubstitutions::built_in().unwrap();
        let rebuilt = parsed
            .inject_with_substitutions(&translation, &CharacterMap::default(), &substitutions)
            .unwrap();
        assert_eq!(rebuilt.substituted_characters, 3);

        let extracted = parse_sdt(
            Path::new("LC01A.SDT"),
            rebuilt.bytes,
            &CharacterMap::default(),
        )
        .unwrap();
        assert_eq!(extracted.translation.entries[0].name.as_deref(), Some("凜"));
        assert_eq!(extracted.translation.entries[0].message, "凜説");
    }

    #[test]
    fn variable_length_name_and_message_remap_target() {
        let source = sample_sdt();
        let old_target = read_u16(&source, 1);
        let parsed = parse_sdt(Path::new("LC01A.SDT"), source, &CharacterMap::default()).unwrap();
        let mut translation = parsed.translation.clone();
        translation.entries[0].name = Some("少女".to_owned());
        translation.entries[0].message = "もっと長い本文<w1><g2>".to_owned();
        let rebuilt = parsed
            .inject(&translation, &CharacterMap::default())
            .unwrap();
        let new_target = read_u16(&rebuilt.bytes, 1);
        assert_ne!(new_target, old_target);

        let extracted = parse_sdt(
            Path::new("LC01A.SDT"),
            rebuilt.bytes,
            &CharacterMap::default(),
        )
        .unwrap();
        assert_eq!(
            extracted.translation.entries[0].name.as_deref(),
            Some("少女")
        );
        assert_eq!(
            extracted.translation.entries[0].message,
            "もっと長い本文<w1><g2>"
        );
    }

    #[test]
    fn rejects_changed_source_validation_fields() {
        let parsed = parse_sdt(
            Path::new("LC01A.SDT"),
            sample_sdt(),
            &CharacterMap::default(),
        )
        .unwrap();
        let mut translation = parsed.translation.clone();
        translation.entries[0].scr_name = Some("別人".to_owned());
        let error = parsed
            .inject(&translation, &CharacterMap::default())
            .unwrap_err();
        assert!(error.to_string().contains("_scr_name mismatch"));
    }

    #[test]
    fn refuses_target_inside_resized_payload() {
        let splices = [Splice {
            start: 10,
            end: 20,
            replacement: vec![0; 12],
            entry_index: 3,
        }];
        let error = remap_offset(15, &splices, "control-flow target").unwrap_err();
        assert!(error.to_string().contains("inside resized entry 3"));
    }

    #[test]
    fn remaps_across_multiple_growing_and_shrinking_payloads() {
        let splices = [
            Splice {
                start: 10,
                end: 14,
                replacement: vec![0; 8],
                entry_index: 1,
            },
            Splice {
                start: 30,
                end: 40,
                replacement: vec![0; 4],
                entry_index: 2,
            },
        ];
        assert_eq!(remap_offset(5, &splices, "target").unwrap(), 5);
        assert_eq!(remap_offset(10, &splices, "target").unwrap(), 10);
        assert_eq!(remap_offset(14, &splices, "target").unwrap(), 18);
        assert_eq!(remap_offset(20, &splices, "target").unwrap(), 24);
        assert_eq!(remap_offset(40, &splices, "target").unwrap(), 38);
        assert_eq!(remap_offset(50, &splices, "target").unwrap(), 48);
    }

    #[test]
    fn leaves_unstructured_brackets_in_message() {
        let split = split_name("ただの【記号】本文");
        assert!(split.name.is_none());
        assert_eq!(split.message, "ただの【記号】本文");
    }

    #[test]
    fn leaves_non_control_prefix_unsplit() {
        let split = split_name("<$>説明【記号】<$>本文");
        assert!(split.name.is_none());
        assert_eq!(split.message, "<$>説明【記号】<$>本文");
    }
}
