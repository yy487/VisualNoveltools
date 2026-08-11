use crate::speaker::SpeakerMap;
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

pub const GSC_HEADER_SIZE: usize = 36;
const GSC_HEADER_SIZE_U32: u32 = 36;
const CHOICE_SIZE: usize = 60;
const RESOURCE_TEXTS: &[&str] = &["grpo", "grpo_bu", "grpo_ex"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GscError(String);

impl GscError {
    fn invalid(message: impl Into<String>) -> Self {
        Self(format!("invalid GSC file: {}", message.into()))
    }

    fn translation(message: impl Into<String>) -> Self {
        Self(format!("invalid GSC translation: {}", message.into()))
    }

    fn encoding(message: impl Into<String>) -> Self {
        Self(format!("CP932 encoding error: {}", message.into()))
    }
}

impl fmt::Display for GscError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for GscError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GscHeader {
    pub file_size: u32,
    pub header_size: u32,
    pub code_size: u32,
    pub text_index_size: u32,
    pub text_pool_size: u32,
    pub sequence_index_size: u32,
    pub sequence_pool_count: u32,
    pub symbol_table_size: u32,
    pub symbol_name_pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextEntry {
    /// Context only. Injection intentionally ignores this field.
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: u32,
    #[serde(rename = "_offset")]
    pub offset: u32,
    #[serde(rename = "_size")]
    pub size: u32,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_policy")]
    pub policy: String,
    #[serde(rename = "_speaker_id", skip_serializing_if = "Option::is_none")]
    pub speaker_id: Option<u16>,
    #[serde(rename = "_inst_offset", skip_serializing_if = "Option::is_none")]
    pub instruction_offset: Option<u32>,
    #[serde(rename = "_opcode", skip_serializing_if = "Option::is_none")]
    pub opcode: Option<String>,
    #[serde(rename = "_target", skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug)]
pub struct GscFile<'a> {
    pub header: GscHeader,
    data: &'a [u8],
    declared_end: usize,
    text_index_start: usize,
    text_pool_start: usize,
    text_pool_end: usize,
    text_offsets: Vec<u32>,
    text_bytes: Vec<&'a [u8]>,
}

#[derive(Debug, Clone)]
struct ChoiceReference {
    instruction_offset: u32,
    target: u32,
}

#[derive(Debug, Clone, Copy)]
struct TextContext<'a> {
    speaker_id: Option<u16>,
    prefix: &'a str,
    body: &'a str,
}

impl<'a> GscFile<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self, GscError> {
        if data.len() < GSC_HEADER_SIZE {
            return Err(GscError::invalid(format!(
                "file is {} bytes, shorter than the {GSC_HEADER_SIZE}-byte header",
                data.len()
            )));
        }
        let header = GscHeader {
            file_size: read_u32(data, 0)?,
            header_size: read_u32(data, 4)?,
            code_size: read_u32(data, 8)?,
            text_index_size: read_u32(data, 12)?,
            text_pool_size: read_u32(data, 16)?,
            sequence_index_size: read_u32(data, 20)?,
            sequence_pool_count: read_u32(data, 24)?,
            symbol_table_size: read_u32(data, 28)?,
            symbol_name_pool_size: read_u32(data, 32)?,
        };
        if header.header_size != GSC_HEADER_SIZE_U32 {
            return Err(GscError::invalid(format!(
                "header size is {}, expected {GSC_HEADER_SIZE}",
                header.header_size
            )));
        }
        let declared_end = usize::try_from(header.file_size)
            .map_err(|_| GscError::invalid("declared size does not fit this platform"))?;
        if declared_end > data.len() {
            return Err(GscError::invalid(format!(
                "declared file size is {}, physical size is {}",
                header.file_size,
                data.len()
            )));
        }
        if !header.text_index_size.is_multiple_of(4) {
            return Err(GscError::invalid("text index size is not divisible by 4"));
        }
        if !header.sequence_index_size.is_multiple_of(4) {
            return Err(GscError::invalid(
                "u16 sequence index size is not divisible by 4",
            ));
        }
        if !header.symbol_table_size.is_multiple_of(4) {
            return Err(GscError::invalid("symbol table size is not divisible by 4"));
        }
        let calculated = calculate_declared_size(&header)?;
        if calculated != declared_end {
            return Err(GscError::invalid(format!(
                "region sizes total {calculated} bytes, header declares {declared_end}"
            )));
        }

        let text_index_start = checked_add(GSC_HEADER_SIZE, header.code_size, "text index")?;
        let text_pool_start = checked_add(text_index_start, header.text_index_size, "text pool")?;
        let text_pool_end = checked_add(text_pool_start, header.text_pool_size, "text pool end")?;
        if text_pool_end > declared_end {
            return Err(GscError::invalid("text pool extends beyond declared file"));
        }

        let text_count = header.text_index_size as usize / 4;
        let mut text_offsets = Vec::with_capacity(text_count);
        let mut text_bytes = Vec::with_capacity(text_count);
        let mut cursor = 0usize;
        for index in 0..text_count {
            let offset = read_u32(data, text_index_start + index * 4)?;
            if offset as usize != cursor {
                return Err(GscError::invalid(format!(
                    "text index {index} points to 0x{offset:x}, expected 0x{cursor:x}"
                )));
            }
            if cursor >= header.text_pool_size as usize {
                return Err(GscError::invalid(format!(
                    "text index {index} starts outside the text pool"
                )));
            }
            let remaining = &data[text_pool_start + cursor..text_pool_end];
            let terminator = remaining
                .iter()
                .position(|byte| *byte == 0)
                .ok_or_else(|| {
                    GscError::invalid(format!("text index {index} is not NUL-terminated"))
                })?;
            text_offsets.push(offset);
            text_bytes.push(&remaining[..terminator]);
            cursor = cursor
                .checked_add(terminator + 1)
                .ok_or_else(|| GscError::invalid("text pool cursor overflow"))?;
        }
        if cursor != header.text_pool_size as usize {
            return Err(GscError::invalid(format!(
                "indexed strings consume {cursor} bytes, text pool declares {}",
                header.text_pool_size
            )));
        }

        Ok(Self {
            header,
            data,
            declared_end,
            text_index_start,
            text_pool_start,
            text_pool_end,
            text_offsets,
            text_bytes,
        })
    }

    #[must_use]
    pub fn text_count(&self) -> usize {
        self.text_bytes.len()
    }

    #[must_use]
    pub fn physical_size(&self) -> usize {
        self.data.len()
    }

    #[must_use]
    pub fn opaque_tail_size(&self) -> usize {
        self.data.len() - self.declared_end
    }

    pub fn extract_entries(
        &self,
        file_name: &str,
        speaker_map: Option<&SpeakerMap>,
    ) -> Result<Vec<TextEntry>, GscError> {
        let choices = self.find_choice_references()?;
        let mut entries = Vec::new();
        for (index, raw) in self.text_bytes.iter().enumerate() {
            let decoded = decode_cp932(raw, index)?;
            if decoded.is_empty() || RESOURCE_TEXTS.contains(&decoded.as_str()) {
                continue;
            }
            let context = text_context(&decoded)?;
            validate_body_controls(context.body, file_name, index)?;
            let scr_msg = context.body.replace("^n", "");
            let choice = choices.get(&index);
            let physical_offset = self
                .text_pool_start
                .checked_add(self.text_offsets[index] as usize)
                .ok_or_else(|| GscError::invalid("physical text offset overflow"))?;

            entries.push(TextEntry {
                name: context
                    .speaker_id
                    .and_then(|id| speaker_map.and_then(|map| map.name(id)))
                    .map(ToOwned::to_owned),
                message: scr_msg.clone(),
                scr_msg,
                file: file_name.to_owned(),
                index: u32::try_from(index)
                    .map_err(|_| GscError::invalid("text index exceeds u32"))?,
                offset: u32::try_from(physical_offset)
                    .map_err(|_| GscError::invalid("physical text offset exceeds u32"))?,
                size: u32::try_from(raw.len())
                    .map_err(|_| GscError::invalid("text byte length exceeds u32"))?,
                entry_type: if choice.is_some() {
                    "choice"
                } else if context.speaker_id.is_some() {
                    "dialogue"
                } else {
                    "message"
                }
                .to_owned(),
                encoding: "cp932".to_owned(),
                policy: "relocate-text-pool".to_owned(),
                speaker_id: context.speaker_id,
                instruction_offset: choice.map(|item| item.instruction_offset),
                opcode: choice.map(|_| "0x000E".to_owned()),
                target: choice.map(|item| format!("0x{:08X}", item.target)),
            });
        }
        Ok(entries)
    }

    pub fn rebuild_from_entries(
        &self,
        file_name: &str,
        translated: &[TextEntry],
    ) -> Result<Vec<u8>, GscError> {
        let expected = self.extract_entries(file_name, None)?;
        let translated_by_index = validate_translations(&expected, translated, file_name)?;

        if expected.iter().all(|source| {
            translated_by_index
                .get(&source.index)
                .is_some_and(|entry| entry.message == source.scr_msg)
        }) {
            return Ok(self.data.to_vec());
        }

        let mut rebuilt_offsets = Vec::with_capacity(self.text_bytes.len());
        let mut rebuilt_pool = Vec::with_capacity(self.header.text_pool_size as usize);
        for (index, original_raw) in self.text_bytes.iter().enumerate() {
            rebuilt_offsets.push(
                u32::try_from(rebuilt_pool.len())
                    .map_err(|_| GscError::translation("rebuilt text offset exceeds u32"))?,
            );
            let index_u32 = u32::try_from(index)
                .map_err(|_| GscError::translation("text index exceeds u32"))?;
            if let Some(entry) = translated_by_index.get(&index_u32) {
                let source = expected
                    .iter()
                    .find(|candidate| candidate.index == index_u32)
                    .ok_or_else(|| GscError::translation("internal source entry mismatch"))?;
                if entry.message == source.scr_msg {
                    rebuilt_pool.extend_from_slice(original_raw);
                } else {
                    let decoded = decode_cp932(original_raw, index)?;
                    let context = text_context(&decoded)?;
                    let encoded_message = encode_message(&entry.message, file_name, index)?;
                    let prefix_len = context.prefix.len();
                    rebuilt_pool.extend_from_slice(&original_raw[..prefix_len]);
                    rebuilt_pool.extend_from_slice(&encoded_message);
                }
            } else {
                rebuilt_pool.extend_from_slice(original_raw);
            }
            rebuilt_pool.push(0);
        }

        let suffix = &self.data[self.text_pool_end..self.declared_end];
        let physical_tail = &self.data[self.declared_end..];
        let new_declared_size = self
            .text_pool_start
            .checked_add(rebuilt_pool.len())
            .and_then(|size| size.checked_add(suffix.len()))
            .ok_or_else(|| GscError::translation("rebuilt declared size overflow"))?;
        let new_declared_u32 = u32::try_from(new_declared_size)
            .map_err(|_| GscError::translation("rebuilt declared size exceeds u32"))?;
        let new_pool_u32 = u32::try_from(rebuilt_pool.len())
            .map_err(|_| GscError::translation("rebuilt text pool exceeds u32"))?;

        let mut output = Vec::with_capacity(new_declared_size + physical_tail.len());
        output.extend_from_slice(&self.data[..self.text_index_start]);
        for offset in rebuilt_offsets {
            output.extend_from_slice(&offset.to_le_bytes());
        }
        output.extend_from_slice(&rebuilt_pool);
        output.extend_from_slice(suffix);
        output.extend_from_slice(physical_tail);
        write_u32(&mut output, 0, new_declared_u32)?;
        write_u32(&mut output, 16, new_pool_u32)?;

        let reparsed = GscFile::parse(&output)?;
        if output[GSC_HEADER_SIZE..self.text_index_start]
            != self.data[GSC_HEADER_SIZE..self.text_index_start]
            || output[reparsed.text_pool_end..reparsed.declared_end] != *suffix
            || output[reparsed.declared_end..] != *physical_tail
        {
            return Err(GscError::translation(format!(
                "{file_name}: a non-text or opaque-tail region changed"
            )));
        }
        Ok(output)
    }

    fn find_choice_references(&self) -> Result<HashMap<usize, ChoiceReference>, GscError> {
        let code = &self.data[GSC_HEADER_SIZE..self.text_index_start];
        let mut choices = HashMap::new();
        if code.len() < CHOICE_SIZE {
            return Ok(choices);
        }
        for offset in (0..=code.len() - CHOICE_SIZE).step_by(2) {
            if read_u16_unchecked(code, offset) != 0x000e {
                continue;
            }
            let count = usize::from(read_u16_unchecked(code, offset + 2) & 0x00ff);
            if !(1..=5).contains(&count) {
                continue;
            }
            let prompt = read_u32_unchecked(code, offset + 4) as usize;
            if prompt >= self.text_count()
                || (0..3).any(|slot| read_u32_unchecked(code, offset + 0x30 + slot * 4) != 0)
            {
                continue;
            }
            let mut references = Vec::with_capacity(count);
            let mut valid = true;
            for slot in 0..5 {
                let target = read_u32_unchecked(code, offset + 8 + slot * 4);
                let text_index = read_u32_unchecked(code, offset + 0x1c + slot * 4) as usize;
                if slot < count {
                    if target as usize >= code.len()
                        || text_index == 0
                        || text_index >= self.text_count()
                    {
                        valid = false;
                        break;
                    }
                    references.push((text_index, target));
                } else if target != 0 || text_index != 0 {
                    valid = false;
                    break;
                }
            }
            if !valid {
                continue;
            }
            let instruction_offset = u32::try_from(GSC_HEADER_SIZE + offset)
                .map_err(|_| GscError::invalid("choice instruction offset exceeds u32"))?;
            for (text_index, target) in references {
                if choices
                    .insert(
                        text_index,
                        ChoiceReference {
                            instruction_offset,
                            target,
                        },
                    )
                    .is_some()
                {
                    return Err(GscError::invalid(format!(
                        "choice text index {text_index} has multiple instruction references"
                    )));
                }
            }
        }
        Ok(choices)
    }
}

#[must_use]
pub fn looks_like_gsc(data: &[u8]) -> bool {
    if data.len() < GSC_HEADER_SIZE {
        return false;
    }
    let header_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let declared = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    header_size == GSC_HEADER_SIZE_U32 && (GSC_HEADER_SIZE..=data.len()).contains(&declared)
}

fn text_context(text: &str) -> Result<TextContext<'_>, GscError> {
    if !text.starts_with("^g") {
        return Ok(TextContext {
            speaker_id: None,
            prefix: "",
            body: text,
        });
    }
    let prefix = text
        .get(..5)
        .ok_or_else(|| GscError::invalid("truncated ^g speaker prefix"))?;
    let digits = &prefix[2..];
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(GscError::invalid(format!(
            "malformed speaker prefix {prefix:?}"
        )));
    }
    let speaker_id = digits
        .parse::<u16>()
        .map_err(|_| GscError::invalid(format!("invalid speaker ID {digits:?}")))?;
    Ok(TextContext {
        speaker_id: Some(speaker_id),
        prefix,
        body: &text[5..],
    })
}

fn validate_body_controls(text: &str, file_name: &str, index: usize) -> Result<(), GscError> {
    let bytes = text.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] == b'^' {
            match bytes.get(cursor + 1) {
                Some(b'n') => cursor += 2,
                Some(code) => {
                    return Err(GscError::invalid(format!(
                        "{file_name} index {index}: unsupported body control ^{}",
                        char::from(*code)
                    )));
                }
                None => {
                    return Err(GscError::invalid(format!(
                        "{file_name} index {index}: dangling ^ control"
                    )));
                }
            }
        } else {
            cursor += 1;
        }
    }
    Ok(())
}

fn encode_message(text: &str, file_name: &str, index: usize) -> Result<Vec<u8>, GscError> {
    if text.contains('\0') {
        return Err(GscError::translation(format!(
            "{file_name} index {index}: message contains NUL"
        )));
    }
    let mut script = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '\r' => {
                if chars.next_if_eq(&'\n').is_none() {
                    return Err(GscError::translation(format!(
                        "{file_name} index {index}: message contains bare carriage return"
                    )));
                }
                script.push_str("^n");
            }
            '\n' => script.push_str("^n"),
            _ => script.push(character),
        }
    }
    validate_translation_controls(&script, file_name, index)?;
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&script);
    if had_errors {
        return Err(GscError::encoding(format!(
            "{file_name} index {index}: message contains characters not encodable as CP932"
        )));
    }
    Ok(encoded.into_owned())
}

fn validate_translation_controls(
    script: &str,
    file_name: &str,
    index: usize,
) -> Result<(), GscError> {
    let bytes = script.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] == b'^' {
            match bytes.get(cursor + 1) {
                Some(b'n') => cursor += 2,
                Some(b'g') => {
                    return Err(GscError::translation(format!(
                        "{file_name} index {index}: structural ^g speaker controls cannot be added to message"
                    )));
                }
                Some(code) => {
                    return Err(GscError::translation(format!(
                        "{file_name} index {index}: malformed or unsupported control ^{}",
                        char::from(*code)
                    )));
                }
                None => {
                    return Err(GscError::translation(format!(
                        "{file_name} index {index}: dangling ^ control"
                    )));
                }
            }
        } else {
            cursor += 1;
        }
    }
    Ok(())
}

fn validate_translations<'a>(
    expected: &[TextEntry],
    translated: &'a [TextEntry],
    file_name: &str,
) -> Result<HashMap<u32, &'a TextEntry>, GscError> {
    if translated.len() != expected.len() {
        return Err(GscError::translation(format!(
            "{file_name}: JSON contains {} entries, expected {}",
            translated.len(),
            expected.len()
        )));
    }
    let mut by_index = HashMap::with_capacity(translated.len());
    for entry in translated {
        if by_index.insert(entry.index, entry).is_some() {
            return Err(GscError::translation(format!(
                "{file_name}: duplicate text index {}",
                entry.index
            )));
        }
    }
    let expected_indices: HashSet<_> = expected.iter().map(|entry| entry.index).collect();
    for source in expected {
        let entry = by_index.get(&source.index).ok_or_else(|| {
            GscError::translation(format!("{file_name}: missing text index {}", source.index))
        })?;
        validate_immutable(source, entry)?;
    }
    if let Some(extra) = by_index
        .keys()
        .find(|index| !expected_indices.contains(index))
    {
        return Err(GscError::translation(format!(
            "{file_name}: unexpected text index {extra}"
        )));
    }
    Ok(by_index)
}

fn validate_immutable(source: &TextEntry, translated: &TextEntry) -> Result<(), GscError> {
    macro_rules! equal {
        ($field:ident) => {
            if source.$field != translated.$field {
                return Err(GscError::translation(format!(
                    "{} index {}: immutable field {} changed",
                    source.file,
                    source.index,
                    stringify!($field)
                )));
            }
        };
    }
    equal!(scr_msg);
    equal!(file);
    equal!(offset);
    equal!(size);
    equal!(entry_type);
    equal!(encoding);
    equal!(policy);
    equal!(speaker_id);
    equal!(instruction_offset);
    equal!(opcode);
    equal!(target);
    // `name` is deliberately not checked: it is non-writable context.
    Ok(())
}

fn decode_cp932(raw: &[u8], index: usize) -> Result<String, GscError> {
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(raw);
    if had_errors {
        return Err(GscError::encoding(format!(
            "text index {index} contains invalid bytes"
        )));
    }
    Ok(decoded.into_owned())
}

fn calculate_declared_size(header: &GscHeader) -> Result<usize, GscError> {
    let fields = [
        u64::from(header.header_size),
        u64::from(header.code_size),
        u64::from(header.text_index_size),
        u64::from(header.text_pool_size),
        u64::from(header.sequence_index_size),
        u64::from(header.sequence_pool_count) * 2,
        u64::from(header.symbol_table_size) * 2,
        u64::from(header.symbol_name_pool_size),
    ];
    let total = fields
        .into_iter()
        .try_fold(0u64, u64::checked_add)
        .ok_or_else(|| GscError::invalid("declared region sizes overflow u64"))?;
    usize::try_from(total).map_err(|_| GscError::invalid("declared size does not fit platform"))
}

fn checked_add(base: usize, size: u32, label: &str) -> Result<usize, GscError> {
    base.checked_add(size as usize)
        .ok_or_else(|| GscError::invalid(format!("{label} offset overflow")))
}

fn read_u16_unchecked(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_u32_unchecked(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32, GscError> {
    let value = data
        .get(offset..offset + 4)
        .ok_or_else(|| GscError::invalid(format!("missing u32 at 0x{offset:x}")))?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn write_u32(data: &mut [u8], offset: usize, value: u32) -> Result<(), GscError> {
    let target = data
        .get_mut(offset..offset + 4)
        .ok_or_else(|| GscError::invalid(format!("missing output u32 at 0x{offset:x}")))?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(text: &str) -> Vec<u8> {
        let (encoded, _, errors) = SHIFT_JIS.encode(text);
        assert!(!errors);
        encoded.into_owned()
    }

    fn fixture() -> Vec<u8> {
        let texts = ["", "^g118本文^n次行", "選択一", "grpo"];
        let mut code = vec![0u8; CHOICE_SIZE];
        code[0..2].copy_from_slice(&0x000eu16.to_le_bytes());
        code[2..4].copy_from_slice(&1u16.to_le_bytes());
        code[4..8].copy_from_slice(&1u32.to_le_bytes());
        code[8..12].copy_from_slice(&2u32.to_le_bytes());
        code[0x1c..0x20].copy_from_slice(&2u32.to_le_bytes());
        let mut offsets = Vec::new();
        let mut pool = Vec::new();
        for text in texts {
            offsets.push(pool.len() as u32);
            pool.extend_from_slice(&encode(text));
            pool.push(0);
        }
        let declared = GSC_HEADER_SIZE + code.len() + offsets.len() * 4 + pool.len();
        let mut output = Vec::new();
        output.extend_from_slice(&(declared as u32).to_le_bytes());
        output.extend_from_slice(&GSC_HEADER_SIZE_U32.to_le_bytes());
        output.extend_from_slice(&(code.len() as u32).to_le_bytes());
        output.extend_from_slice(&((offsets.len() * 4) as u32).to_le_bytes());
        output.extend_from_slice(&(pool.len() as u32).to_le_bytes());
        output.extend_from_slice(&[0u8; 16]);
        output.extend_from_slice(&code);
        for offset in offsets {
            output.extend_from_slice(&offset.to_le_bytes());
        }
        output.extend_from_slice(&pool);
        output.extend_from_slice(b"OPAQUE-TAIL\0\xff");
        output
    }

    #[test]
    fn accepts_physical_tail_and_removes_forced_breaks_from_json() {
        let input = fixture();
        let gsc = GscFile::parse(&input).unwrap();
        assert_eq!(gsc.opaque_tail_size(), 13);
        let entries = gsc.extract_entries("test.gsc", None).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].speaker_id, Some(118));
        assert_eq!(entries[0].scr_msg, "本文次行");
        assert_eq!(entries[1].entry_type, "choice");
    }

    #[test]
    fn unchanged_body_is_byte_exact_even_if_name_changes() {
        let input = fixture();
        let gsc = GscFile::parse(&input).unwrap();
        let mut entries = gsc.extract_entries("test.gsc", None).unwrap();
        entries[0].name = Some("绝不会写入".to_owned());
        let output = gsc.rebuild_from_entries("test.gsc", &entries).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn edited_body_relocates_pool_and_preserves_prefix_and_tail() {
        let input = fixture();
        let gsc = GscFile::parse(&input).unwrap();
        let old_tail = input[gsc.declared_end..].to_vec();
        let mut entries = gsc.extract_entries("test.gsc", None).unwrap();
        entries[0].message = "新本文".to_owned();
        let output = gsc.rebuild_from_entries("test.gsc", &entries).unwrap();
        let rebuilt = GscFile::parse(&output).unwrap();
        assert_eq!(&output[rebuilt.declared_end..], old_tail);
        assert_eq!(
            decode_cp932(rebuilt.text_bytes[1], 1).unwrap(),
            "^g118新本文"
        );
    }
}
