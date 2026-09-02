use crate::format::{CodeRegion, Command, ParameterKind, TobFile};
use crate::Result;
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Range;

pub const TRANSLATION_FORMAT: &str = "ORETUBAR TOB0 translation";
pub const TRANSLATION_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationFile {
    pub format: String,
    pub version: u32,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: u32,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_inst_offset", skip_serializing_if = "Option::is_none")]
    pub instruction_offset: Option<u32>,
    #[serde(rename = "_opcode", skip_serializing_if = "Option::is_none")]
    pub opcode: Option<u32>,
    #[serde(rename = "_table_index", skip_serializing_if = "Option::is_none")]
    pub table_index: Option<usize>,
    #[serde(rename = "_voice", skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scr_msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scr_msg_parts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_parts: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSlot {
    pub range: Range<usize>,
    pub command_start: Option<usize>,
    pub length_pos: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct LocatedEntry {
    pub json: TranslationEntry,
    pub slots: Vec<TextSlot>,
    pub name_slot: Option<TextSlot>,
}

#[derive(Debug, Clone)]
pub struct Replacement {
    pub slot: TextSlot,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
struct Candidate {
    sort_offset: usize,
    entry_type: &'static str,
    slots: Vec<TextSlot>,
    instruction_offset: Option<u32>,
    opcode: Option<u32>,
    table_index: Option<usize>,
    name: Option<String>,
    name_slot: Option<TextSlot>,
    voice: Option<String>,
}

pub fn extract(file_name: &str, file: &TobFile) -> Result<TranslationFile> {
    let located = locate_entries(file_name, file)?;
    Ok(TranslationFile {
        format: TRANSLATION_FORMAT.to_string(),
        version: TRANSLATION_VERSION,
        file: file_name.to_string(),
        encoding: "cp932".to_string(),
        entries: located.into_iter().map(|entry| entry.json).collect(),
    })
}

pub fn locate_entries(file_name: &str, file: &TobFile) -> Result<Vec<LocatedEntry>> {
    let mut candidates = Vec::new();
    for command in file.regions.iter().filter_map(|region| match region {
        CodeRegion::Command(command) => Some(command),
        _ => None,
    }) {
        match command.opcode {
            18 => {
                if let Some(slot) = string_slot(command, 0) {
                    candidates.push(Candidate {
                        sort_offset: slot.range.start,
                        entry_type: "title",
                        slots: vec![slot],
                        instruction_offset: Some(file.code_offset(command.start)?),
                        opcode: Some(command.opcode),
                        table_index: table_index_for(file, command.start),
                        name: None,
                        name_slot: None,
                        voice: None,
                    });
                }
            }
            127 => {
                if let Some(slot) = string_slot(command, 6) {
                    candidates.push(Candidate {
                        sort_offset: slot.range.start,
                        entry_type: "summary",
                        slots: vec![slot],
                        instruction_offset: Some(file.code_offset(command.start)?),
                        opcode: Some(command.opcode),
                        table_index: table_index_for(file, command.start),
                        name: None,
                        name_slot: None,
                        voice: None,
                    });
                }
            }
            13 => {
                let slots = command
                    .parameters
                    .iter()
                    .enumerate()
                    .skip(1)
                    .filter_map(|(index, _)| string_slot(command, index))
                    .collect::<Vec<_>>();
                if !slots.is_empty() {
                    candidates.push(Candidate {
                        sort_offset: command.start,
                        entry_type: "selection",
                        slots,
                        instruction_offset: Some(file.code_offset(command.start)?),
                        opcode: Some(command.opcode),
                        table_index: table_index_for(file, command.start),
                        name: None,
                        name_slot: None,
                        voice: None,
                    });
                }
            }
            _ => {}
        }
    }

    let mut text_by_table = BTreeMap::<usize, Vec<Range<usize>>>::new();
    for span in file.regions.iter().filter_map(|region| match region {
        CodeRegion::Text(span) => Some(span),
        _ => None,
    }) {
        let Some(table_index) = table_index_for(file, span.start) else {
            return Err(format!(
                "display text at code offset 0x{:x} precedes the first offset-table entry",
                span.start - file.code_start
            ));
        };
        text_by_table
            .entry(table_index)
            .or_default()
            .push(span.clone());
    }
    for (table_index, spans) in text_by_table {
        let first = spans.first().expect("text group is not empty").start;
        let interval_start = file.code_start + file.offsets[table_index] as usize;
        let interval_end = file
            .offsets
            .get(table_index + 1)
            .map(|target| file.code_start + *target as usize)
            .unwrap_or(file.opaque_tail.start);
        let speaker_command = file
            .regions
            .iter()
            .filter_map(|region| match region {
                CodeRegion::Command(command)
                    if command.opcode == 25
                        && command.start >= interval_start
                        && command.start < first =>
                {
                    Some(command)
                }
                _ => None,
            })
            .next_back();
        let (name, name_slot, voice) = if let Some(command) = speaker_command {
            let name_slot = string_slot(command, 0);
            let name = name_slot
                .as_ref()
                .map(|slot| decode_cp932(&file.bytes[slot.range.clone()]))
                .transpose()?
                .filter(|value| !value.is_empty());
            let voice = string_slot(command, 1)
                .map(|slot| decode_cp932(&file.bytes[slot.range]))
                .transpose()?
                .filter(|value| !value.is_empty());
            (name, name_slot, voice)
        } else {
            (None, None, None)
        };
        if spans.iter().any(|span| span.end > interval_end) {
            return Err(format!(
                "text group for offset-table entry {table_index} crosses the next entry"
            ));
        }
        candidates.push(Candidate {
            sort_offset: first,
            entry_type: if name.is_some() {
                "dialogue"
            } else {
                "narration"
            },
            slots: spans
                .into_iter()
                .map(|range| TextSlot {
                    range,
                    command_start: None,
                    length_pos: None,
                })
                .collect(),
            instruction_offset: None,
            opcode: None,
            table_index: Some(table_index),
            name,
            name_slot,
            voice,
        });
    }

    candidates.sort_by_key(|candidate| candidate.sort_offset);
    let mut entries = Vec::with_capacity(candidates.len());
    for (index, candidate) in candidates.into_iter().enumerate() {
        let parts = candidate
            .slots
            .iter()
            .map(|slot| decode_cp932(&file.bytes[slot.range.clone()]))
            .collect::<Result<Vec<_>>>()?;
        let offset = file.code_offset(candidate.slots[0].range.start)?;
        let size = candidate.slots.iter().map(|slot| slot.range.len()).sum();
        let (scr_msg, message, scr_msg_parts, message_parts) = if parts.len() == 1 {
            let value = parts[0].clone();
            (Some(value.clone()), Some(value), None, None)
        } else {
            (None, None, Some(parts.clone()), Some(parts))
        };
        let source_name = candidate.name.clone();
        entries.push(LocatedEntry {
            json: TranslationEntry {
                file: file_name.to_string(),
                index,
                offset,
                size,
                entry_type: candidate.entry_type.to_string(),
                encoding: "cp932".to_string(),
                instruction_offset: candidate.instruction_offset,
                opcode: candidate.opcode,
                table_index: candidate.table_index,
                voice: candidate.voice,
                name: candidate.name,
                source_name,
                scr_msg,
                message,
                scr_msg_parts,
                message_parts,
            },
            slots: candidate.slots,
            name_slot: candidate.name_slot,
        });
    }
    Ok(entries)
}

pub fn prepare_replacements(
    file_name: &str,
    file: &TobFile,
    translation: &TranslationFile,
    names_writable: bool,
) -> Result<Vec<Replacement>> {
    if translation.format != TRANSLATION_FORMAT || translation.version != TRANSLATION_VERSION {
        return Err("translation JSON format/version does not match this tool".to_string());
    }
    if translation.file != file_name || !translation.encoding.eq_ignore_ascii_case("cp932") {
        return Err(
            "translation JSON file or encoding metadata does not match the TOB".to_string(),
        );
    }
    let source = locate_entries(file_name, file)?;
    if source.len() != translation.entries.len() {
        return Err(format!(
            "translation entry count is {}, source TOB has {}",
            translation.entries.len(),
            source.len()
        ));
    }
    let mut replacements = Vec::new();
    for (located, translated) in source.iter().zip(&translation.entries) {
        validate_entry_metadata(&located.json, translated)?;
        match (
            &located.json.scr_msg,
            &translated.scr_msg,
            &translated.message,
        ) {
            (Some(source_text), Some(json_source), Some(message)) => {
                if source_text != json_source {
                    return Err(format!("entry {} scr_msg was modified", translated.index));
                }
                replacements.push(Replacement {
                    slot: located.slots[0].clone(),
                    bytes: encode_cp932(message)
                        .map_err(|error| format!("entry {} message: {error}", translated.index))?,
                });
            }
            (None, None, None) => {
                let source_parts = translated.scr_msg_parts.as_ref().ok_or_else(|| {
                    format!("entry {} is missing scr_msg_parts", translated.index)
                })?;
                if located.json.scr_msg_parts.as_ref() != Some(source_parts) {
                    return Err(format!(
                        "entry {} scr_msg_parts were modified",
                        translated.index
                    ));
                }
                let message_parts = translated.message_parts.as_ref().ok_or_else(|| {
                    format!("entry {} is missing message_parts", translated.index)
                })?;
                if message_parts.len() != located.slots.len() {
                    return Err(format!(
                        "entry {} message_parts count cannot change",
                        translated.index
                    ));
                }
                for (slot, message) in located.slots.iter().zip(message_parts) {
                    replacements.push(Replacement {
                        slot: slot.clone(),
                        bytes: encode_cp932(message).map_err(|error| {
                            format!("entry {} message_parts: {error}", translated.index)
                        })?,
                    });
                }
            }
            _ => {
                return Err(format!(
                    "entry {} mixes single-part and multipart text fields",
                    translated.index
                ))
            }
        }
        if translated.source_name != located.json.source_name {
            return Err(format!("entry {} _scr_name was modified", translated.index));
        }
        if translated.name != located.json.name {
            if !names_writable {
                return Err(format!(
                    "entry {} changes name, but this project currently treats names as read-only",
                    translated.index
                ));
            }
            let slot = located.name_slot.clone().ok_or_else(|| {
                format!(
                    "entry {} has no writable source name slot",
                    translated.index
                )
            })?;
            let name = translated
                .name
                .as_ref()
                .ok_or_else(|| format!("entry {} cannot remove its name", translated.index))?;
            let encoded = encode_cp932(name)
                .map_err(|error| format!("entry {} name: {error}", translated.index))?;
            if encoded.len() > 16 {
                return Err(format!(
                    "entry {} name is {} bytes; the executable copies at most 16",
                    translated.index,
                    encoded.len()
                ));
            }
            replacements.push(Replacement {
                slot,
                bytes: encoded,
            });
        }
    }
    Ok(replacements)
}

fn validate_entry_metadata(source: &TranslationEntry, translated: &TranslationEntry) -> Result<()> {
    if source.file != translated.file
        || source.index != translated.index
        || source.offset != translated.offset
        || source.size != translated.size
        || source.entry_type != translated.entry_type
        || source.encoding != translated.encoding
        || source.instruction_offset != translated.instruction_offset
        || source.opcode != translated.opcode
        || source.table_index != translated.table_index
        || source.voice != translated.voice
    {
        return Err(format!(
            "entry {} source metadata does not match the TOB",
            translated.index
        ));
    }
    Ok(())
}

fn table_index_for(file: &TobFile, absolute: usize) -> Option<usize> {
    let relative = absolute.checked_sub(file.code_start)?;
    file.offsets
        .partition_point(|target| (*target as usize) <= relative)
        .checked_sub(1)
}

fn string_slot(command: &Command, parameter_index: usize) -> Option<TextSlot> {
    let parameter = command.parameters.get(parameter_index)?;
    if parameter.kind != ParameterKind::String {
        return None;
    }
    Some(TextSlot {
        range: parameter.string.clone()?,
        command_start: Some(command.start),
        length_pos: parameter.length_pos,
    })
}

fn decode_cp932(bytes: &[u8]) -> Result<String> {
    let (text, _, errors) = SHIFT_JIS.decode(bytes);
    if errors {
        return Err("invalid CP932 text".to_string());
    }
    let text = text.into_owned();
    let (roundtrip, _, encode_errors) = SHIFT_JIS.encode(&text);
    if encode_errors || roundtrip.as_ref() != bytes {
        return Err("CP932 text does not round-trip to its original bytes".to_string());
    }
    Ok(text)
}

fn encode_cp932(text: &str) -> Result<Vec<u8>> {
    if text.chars().any(|character| {
        character == '\0' || character == '\r' || (character.is_control() && character != '\n')
    }) {
        return Err("NUL, CR, and control characters other than LF are not allowed".to_string());
    }
    let (bytes, _, errors) = SHIFT_JIS.encode(text);
    if errors {
        let invalid = text
            .chars()
            .filter(|character| {
                let value = character.to_string();
                let (_, _, error) = SHIFT_JIS.encode(&value);
                error
            })
            .map(|character| format!("{character} (U+{:04X})", character as u32))
            .collect::<Vec<_>>();
        return Err(format!(
            "characters are not representable in CP932: {}",
            invalid.join(", ")
        ));
    }
    Ok(bytes.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unencodable_text_with_character_details() {
        let error = encode_cp932("测试😀").expect_err("Chinese and emoji are not CP932");
        assert!(error.contains("U+"));
    }
}
