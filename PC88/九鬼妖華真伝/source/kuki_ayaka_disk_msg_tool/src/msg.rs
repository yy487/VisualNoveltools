use anyhow::{bail, Context, Result};
use encoding_rs::SHIFT_JIS;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Range;

pub const MAX_MSG_SIZE: usize = 0x1000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsgFile {
    pub body_offset: u16,
    pub pointer_block_order: Vec<usize>,
    pub blocks: Vec<MsgBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsgBlock {
    pub original_offset: u16,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MsgFragment {
    Text { offset: usize, text: String },
    Control { offset: usize, value: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsgInspection {
    pub block_count: usize,
    pub pointer_count: usize,
    pub text_fragments: usize,
    pub control_counts: BTreeMap<u8, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsgDisplaySegment {
    pub block_index: usize,
    pub segment_index: usize,
    pub block_relative_offset: usize,
    pub raw_size: usize,
    pub name: Option<String>,
    pub scr_msg: String,
    pub has_editable_text: bool,
    prefix: Vec<u8>,
    name_separator: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedInitialName {
    name: String,
    message_start: usize,
    separator: Vec<u8>,
}

impl MsgFile {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 3 {
            bail!("MSG file is too short");
        }
        let body_offset = read_u16(bytes, 0)?;
        let body = usize::from(body_offset);
        if body < 2 || body > bytes.len() || body % 2 != 0 {
            bail!("invalid MSG body offset 0x{body:04X}");
        }

        let mut pointers = Vec::new();
        for offset in (2..body).step_by(2) {
            pointers.push(read_u16(bytes, offset)?);
        }

        let mut blocks = Vec::new();
        let mut cursor = body;
        while cursor < bytes.len() {
            let relative_end = bytes[cursor..]
                .iter()
                .position(|byte| *byte == 0)
                .with_context(|| format!("unterminated MSG block at 0x{cursor:04X}"))?;
            let end = cursor + relative_end;
            blocks.push(MsgBlock {
                original_offset: u16::try_from(cursor).context("MSG block offset exceeds u16")?,
                bytes: bytes[cursor..end].to_vec(),
            });
            cursor = end + 1;
        }
        let by_offset: HashMap<u16, usize> = blocks
            .iter()
            .enumerate()
            .map(|(index, block)| (block.original_offset, index))
            .collect();
        if by_offset.len() != blocks.len() {
            bail!("duplicate physical MSG block offset");
        }

        let mut pointer_block_order = Vec::with_capacity(pointers.len());
        let mut seen = HashSet::new();
        for pointer in pointers {
            let block_index = *by_offset
                .get(&pointer)
                .with_context(|| format!("MSG pointer 0x{pointer:04X} is not a block start"))?;
            if block_index == 0 {
                bail!("MSG pointer table redundantly references the implicit first block");
            }
            if !seen.insert(block_index) {
                bail!("duplicate MSG pointer to block {block_index}");
            }
            pointer_block_order.push(block_index);
        }
        let parsed = Self {
            body_offset,
            pointer_block_order,
            blocks,
        };
        if parsed.rebuild()? != bytes {
            bail!("MSG parse/rebuild is not byte-exact");
        }
        Ok(parsed)
    }

    pub fn rebuild(&self) -> Result<Vec<u8>> {
        let expected_body = 2 + self.pointer_block_order.len() * 2;
        if usize::from(self.body_offset) != expected_body {
            bail!("MSG body offset no longer matches pointer table size");
        }
        if self.blocks.is_empty() {
            bail!("MSG contains no physical blocks");
        }

        let mut offsets = Vec::with_capacity(self.blocks.len());
        let mut cursor = expected_body;
        for block in &self.blocks {
            offsets.push(u16::try_from(cursor).context("rebuilt MSG offset exceeds u16")?);
            cursor = cursor
                .checked_add(block.bytes.len() + 1)
                .context("rebuilt MSG size overflow")?;
        }
        if cursor > MAX_MSG_SIZE {
            bail!(
                "rebuilt MSG size {cursor} exceeds the runtime F000h buffer limit of {MAX_MSG_SIZE} bytes"
            );
        }

        let mut output = Vec::with_capacity(cursor);
        output.extend_from_slice(&self.body_offset.to_le_bytes());
        for block_index in &self.pointer_block_order {
            let offset = offsets
                .get(*block_index)
                .context("MSG pointer references a missing block")?;
            output.extend_from_slice(&offset.to_le_bytes());
        }
        for block in &self.blocks {
            output.extend_from_slice(&block.bytes);
            output.push(0);
        }
        Ok(output)
    }

    pub fn fragments(&self, block_index: usize) -> Result<Vec<MsgFragment>> {
        let block = self
            .blocks
            .get(block_index)
            .with_context(|| format!("missing MSG block {block_index}"))?;
        tokenize_block(&block.bytes)
    }

    pub fn logical_block_order(&self) -> Vec<usize> {
        let mut order = Vec::with_capacity(self.blocks.len());
        order.push(0);
        order.extend(self.pointer_block_order.iter().copied());
        order
    }

    pub fn inspect(&self) -> Result<MsgInspection> {
        let mut text_fragments = 0usize;
        let mut control_counts = BTreeMap::new();
        for block_index in 0..self.blocks.len() {
            for fragment in self.fragments(block_index)? {
                match fragment {
                    MsgFragment::Text { .. } => text_fragments += 1,
                    MsgFragment::Control { value, .. } => {
                        *control_counts.entry(value).or_insert(0) += 1;
                    }
                }
            }
        }
        Ok(MsgInspection {
            block_count: self.blocks.len(),
            pointer_count: self.pointer_block_order.len(),
            text_fragments,
            control_counts,
        })
    }

    pub fn render_block_for_inspection(&self, block_index: usize) -> Result<String> {
        let mut output = String::new();
        for fragment in self.fragments(block_index)? {
            match fragment {
                MsgFragment::Text { text, .. } => output.push_str(&text),
                MsgFragment::Control { value, .. } => {
                    use std::fmt::Write;
                    let _ = write!(output, "[CTRL:{value:02X}]");
                }
            }
        }
        Ok(output)
    }

    pub fn display_segments(&self, block_index: usize) -> Result<Vec<MsgDisplaySegment>> {
        let block = self
            .blocks
            .get(block_index)
            .with_context(|| format!("missing MSG block {block_index}"))?;
        let ranges = split_display_ranges(&block.bytes)?;
        ranges
            .into_iter()
            .enumerate()
            .map(|(segment_index, range)| {
                parse_display_segment(block_index, segment_index, range, &block.bytes)
            })
            .collect()
    }
}

impl MsgDisplaySegment {
    pub fn rebuild(&self, name: Option<&str>, message: &str) -> Result<Vec<u8>> {
        let mut encoder = |value: &str, role: &str| encode_plain_cp932(value, role);
        self.rebuild_with_encoder(name, message, &mut encoder)
    }

    pub fn rebuild_with_encoder(
        &self,
        name: Option<&str>,
        message: &str,
        encoder: &mut dyn FnMut(&str, &str) -> Result<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        let mut output = self.prefix.clone();
        match (&self.name, name) {
            (Some(_), Some(name)) => {
                output.extend_from_slice(&encoder(name, "name")?);
                output.extend_from_slice(&self.name_separator);
            }
            (Some(_), None) => bail!("source segment has a name but translated name is missing"),
            (None, Some(_)) => bail!("source segment has no writable name"),
            (None, None) => {}
        }
        output.extend_from_slice(&decode_editable_text_with(message, encoder)?);
        Ok(output)
    }
}

pub fn tokenize_block(bytes: &[u8]) -> Result<Vec<MsgFragment>> {
    let mut output = Vec::new();
    let mut text_start = 0;
    let mut cursor = 0;

    let flush_text = |start: usize, end: usize, output: &mut Vec<MsgFragment>| -> Result<()> {
        if start == end {
            return Ok(());
        }
        let raw = &bytes[start..end];
        let decoded = SHIFT_JIS
            .decode_without_bom_handling_and_without_replacement(raw)
            .with_context(|| {
                let hex = raw
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect::<String>();
                format!("invalid CP932 text at block offset 0x{start:04X}: {hex}")
            })?;
        let (encoded, _, had_errors) = SHIFT_JIS.encode(&decoded);
        if had_errors || encoded.as_ref() != raw {
            bail!("CP932 round-trip mismatch at block offset 0x{start:04X}");
        }
        output.push(MsgFragment::Text {
            offset: start,
            text: decoded.into_owned(),
        });
        Ok(())
    };

    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if is_sjis_lead(byte) {
            if cursor + 1 >= bytes.len() || !is_sjis_trail(bytes[cursor + 1]) {
                bail!("invalid CP932 lead byte at block offset 0x{cursor:04X}");
            }
            cursor += 2;
            continue;
        }
        if is_control(byte) {
            flush_text(text_start, cursor, &mut output)?;
            output.push(MsgFragment::Control {
                offset: cursor,
                value: byte,
            });
            cursor += 1;
            text_start = cursor;
            continue;
        }
        cursor += 1;
    }
    flush_text(text_start, cursor, &mut output)?;
    Ok(output)
}

pub fn render_editable_bytes(bytes: &[u8]) -> Result<String> {
    let mut output = String::new();
    for fragment in tokenize_block(bytes)? {
        match fragment {
            MsgFragment::Text { text, .. } => output.push_str(&text.replace('[', "[[")),
            MsgFragment::Control { value, .. } => {
                use std::fmt::Write;
                let _ = write!(output, "[{value:02X}]");
            }
        }
    }
    Ok(output)
}

pub fn decode_editable_text(value: &str) -> Result<Vec<u8>> {
    let mut encoder = |plain: &str, role: &str| encode_plain_cp932(plain, role);
    decode_editable_text_with(value, &mut encoder)
}

pub fn decode_editable_text_with(
    value: &str,
    encoder: &mut dyn FnMut(&str, &str) -> Result<Vec<u8>>,
) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut plain = String::new();
    let mut chars = value.char_indices().peekable();
    while let Some((offset, character)) = chars.next() {
        if character != '[' {
            if matches!(character, '\0' | '\r' | '\n') || character.is_control() {
                bail!(
                    "message contains forbidden U+{:04X} at character offset {offset}; use [HH] control syntax",
                    u32::from(character)
                );
            }
            plain.push(character);
            continue;
        }
        if chars.peek().is_some_and(|(_, next)| *next == '[') {
            chars.next();
            plain.push('[');
            continue;
        }

        let remaining = &value[offset..];
        let token = remaining.as_bytes();
        if token.len() >= 4
            && token[3] == b']'
            && token[1].is_ascii_hexdigit()
            && token[2].is_ascii_hexdigit()
        {
            flush_plain_text_with(&mut plain, &mut output, "message", encoder)?;
            let control =
                u8::from_str_radix(&remaining[1..3], 16).context("failed to parse control byte")?;
            if control == 0x00 {
                bail!("[00] is the structural MSG record terminator and is not editable");
            }
            if control == 0x0E {
                bail!("[0E] is managed automatically as the display-segment delimiter");
            }
            if !is_control(control) {
                bail!("[{control:02X}] is not a control byte supported by this MSG format");
            }
            chars.next();
            chars.next();
            chars.next();
            output.push(control);
            continue;
        }
        bail!(
            "malformed control token at character offset {offset}; use [HH] or [[ for a literal ["
        );
    }
    flush_plain_text_with(&mut plain, &mut output, "message", encoder)?;
    Ok(output)
}

fn split_display_ranges(bytes: &[u8]) -> Result<Vec<Range<usize>>> {
    let mut ranges = Vec::new();
    let mut start = 0usize;
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if is_sjis_lead(byte) {
            if cursor + 1 >= bytes.len() || !is_sjis_trail(bytes[cursor + 1]) {
                bail!("invalid CP932 lead byte at block offset 0x{cursor:04X}");
            }
            cursor += 2;
            continue;
        }
        if byte == 0x0E {
            ranges.push(start..cursor);
            start = cursor + 1;
        }
        cursor += 1;
    }
    ranges.push(start..bytes.len());
    Ok(ranges)
}

fn parse_display_segment(
    block_index: usize,
    segment_index: usize,
    range: Range<usize>,
    block_bytes: &[u8],
) -> Result<MsgDisplaySegment> {
    let raw = &block_bytes[range.clone()];
    let content_start = leading_prefix_len(raw)?;
    let (name, message_start, name_separator) = match parse_initial_name(raw, content_start)? {
        Some(parsed) => (Some(parsed.name), parsed.message_start, parsed.separator),
        None => (None, content_start, Vec::new()),
    };
    let message_bytes = &raw[message_start..];
    let scr_msg = render_editable_bytes(message_bytes)?;
    let has_editable_text = tokenize_block(message_bytes)?.iter().any(|fragment| {
        matches!(fragment, MsgFragment::Text { text, .. } if text.chars().any(|character| !character.is_whitespace()))
    });
    Ok(MsgDisplaySegment {
        block_index,
        segment_index,
        block_relative_offset: range.start,
        raw_size: raw.len(),
        name,
        scr_msg,
        has_editable_text,
        prefix: raw[..content_start].to_vec(),
        name_separator,
    })
}

fn leading_prefix_len(bytes: &[u8]) -> Result<usize> {
    let mut cursor = 0usize;
    loop {
        if cursor >= bytes.len() {
            return Ok(cursor);
        }
        if bytes[cursor] == b' ' {
            cursor += 1;
            continue;
        }
        if cursor + 1 < bytes.len() && bytes[cursor..cursor + 2] == [0x81, 0x40] {
            cursor += 2;
            continue;
        }
        if is_sjis_lead(bytes[cursor]) {
            if cursor + 1 >= bytes.len() || !is_sjis_trail(bytes[cursor + 1]) {
                bail!("invalid CP932 lead byte at segment offset 0x{cursor:04X}");
            }
            return Ok(cursor);
        }
        if is_prefix_control(bytes[cursor]) {
            cursor += 1;
            continue;
        }
        return Ok(cursor);
    }
}

fn is_prefix_control(value: u8) -> bool {
    value == 0x10 || value == 0xA0 || value >= 0xF8
}

fn parse_initial_name(bytes: &[u8], content_start: usize) -> Result<Option<ParsedInitialName>> {
    let mut cursor = content_start;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if is_sjis_lead(byte) {
            if cursor + 1 >= bytes.len() || !is_sjis_trail(bytes[cursor + 1]) {
                bail!("invalid CP932 lead byte at segment offset 0x{cursor:04X}");
            }
            if matches!(&bytes[cursor..cursor + 2], [0x81, 0x75] | [0x81, 0x69]) {
                let name_end = trim_trailing_horizontal_space(bytes, content_start, cursor);
                if name_end == content_start {
                    return Ok(None);
                }
                let separator = bytes[name_end..cursor].to_vec();
                let name = decode_cp932_strict(&bytes[content_start..name_end], content_start)?;
                if is_name_candidate(&name)
                    && (!separator.is_empty() || is_confirmed_unspaced_speaker(&name))
                {
                    return Ok(Some(ParsedInitialName {
                        name,
                        message_start: cursor,
                        separator,
                    }));
                }
                return Ok(None);
            }
            cursor += 2;
            continue;
        }
        if is_control(byte) {
            return Ok(None);
        }
        cursor += 1;
    }
    Ok(None)
}

fn is_confirmed_unspaced_speaker(value: &str) -> bool {
    matches!(
        value,
        "妖華"
            | "清美"
            | "いつみ"
            | "魔牙"
            | "美牙"
            | "九鬼"
            | "近従"
            | "観世音"
            | "館長"
            | "Ａ子"
            | "Ｂ子"
    )
}

fn trim_trailing_horizontal_space(bytes: &[u8], start: usize, mut end: usize) -> usize {
    loop {
        if end > start && bytes[end - 1] == b' ' {
            end -= 1;
            continue;
        }
        if end >= start + 2 && bytes[end - 2..end] == [0x81, 0x40] {
            end -= 2;
            continue;
        }
        return end;
    }
}

fn is_name_candidate(value: &str) -> bool {
    let count = value.chars().count();
    count > 0
        && count <= 24
        && !value.chars().any(|character| {
            character.is_whitespace()
                || matches!(
                    character,
                    '「' | '」'
                        | '（'
                        | '）'
                        | '＜'
                        | '＞'
                        | '。'
                        | '、'
                        | '！'
                        | '？'
                        | '!'
                        | '?'
                        | ','
                        | '.'
                        | ':'
                        | ';'
                )
        })
}

fn encode_plain_cp932(value: &str, role: &str) -> Result<Vec<u8>> {
    if value.chars().any(|character| character.is_control()) {
        bail!("{role} contains a forbidden Unicode control character");
    }
    let (encoded, _, had_errors) = SHIFT_JIS.encode(value);
    if had_errors {
        let unsupported = value
            .chars()
            .filter(|character| {
                let value = character.to_string();
                SHIFT_JIS.encode(&value).2
            })
            .map(|character| format!("{character} (U+{:04X})", u32::from(character)))
            .collect::<Vec<_>>()
            .join(", ");
        bail!("{role} contains characters not encodable as CP932: {unsupported}");
    }
    if encoded.contains(&0x00) {
        bail!("{role} encodes to the structural NUL terminator");
    }
    Ok(encoded.into_owned())
}

pub fn encode_plain_cp932_text(value: &str, role: &str) -> Result<Vec<u8>> {
    encode_plain_cp932(value, role)
}

fn flush_plain_text_with(
    plain: &mut String,
    output: &mut Vec<u8>,
    role: &str,
    encoder: &mut dyn FnMut(&str, &str) -> Result<Vec<u8>>,
) -> Result<()> {
    if !plain.is_empty() {
        output.extend_from_slice(&encoder(plain, role)?);
        plain.clear();
    }
    Ok(())
}

fn decode_cp932_strict(bytes: &[u8], offset: usize) -> Result<String> {
    let decoded = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .with_context(|| format!("invalid CP932 text at block offset 0x{offset:04X}"))?;
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&decoded);
    if had_errors || encoded.as_ref() != bytes {
        bail!("CP932 round-trip mismatch at block offset 0x{offset:04X}");
    }
    Ok(decoded.into_owned())
}

fn is_sjis_lead(byte: u8) -> bool {
    (0x81..=0x9F).contains(&byte) || (0xE0..=0xF7).contains(&byte)
}

fn is_sjis_trail(byte: u8) -> bool {
    (0x40..=0x7E).contains(&byte) || (0x80..=0xFC).contains(&byte)
}

fn is_control(byte: u8) -> bool {
    byte <= 0x1F || matches!(byte, 0x7F | 0xA0) || byte >= 0xF8
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let raw = bytes
        .get(offset..offset + 2)
        .with_context(|| format!("truncated u16 at 0x{offset:04X}"))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_out_of_order_pointer_table_and_round_trips() {
        let bytes = [
            0x08, 0x00, 0x0E, 0x00, 0x0B, 0x00, 0x11, 0x00, b'A', b'B', 0x00, b'C', b'D', 0x00,
            b'E', b'F', 0x00, b'G', 0x00,
        ];
        let parsed = MsgFile::parse(&bytes).unwrap();
        assert_eq!(parsed.pointer_block_order, vec![2, 1, 3]);
        assert_eq!(parsed.rebuild().unwrap(), bytes);
    }

    #[test]
    fn distinguishes_sjis_trail_fc_from_control_fc() {
        let fragments = tokenize_block(&[0xFC, 0x94, 0xFC, 0x0E, b'A']).unwrap();
        assert_eq!(
            fragments,
            vec![
                MsgFragment::Control {
                    offset: 0,
                    value: 0xFC
                },
                MsgFragment::Text {
                    offset: 1,
                    text: "美".to_owned()
                },
                MsgFragment::Control {
                    offset: 3,
                    value: 0x0E
                },
                MsgFragment::Text {
                    offset: 4,
                    text: "A".to_owned()
                }
            ]
        );
    }

    #[test]
    fn preserves_unreferenced_and_trailing_empty_blocks() {
        let bytes = [
            0x04, 0x00, 0x08, 0x00, b'A', 0x00, b'B', 0x00, b'C', 0x00, 0x00,
        ];
        let parsed = MsgFile::parse(&bytes).unwrap();
        assert_eq!(parsed.pointer_block_order, vec![2]);
        assert_eq!(parsed.blocks.len(), 4);
        assert_eq!(parsed.rebuild().unwrap(), bytes);
    }

    #[test]
    fn variable_length_rebuild_updates_out_of_order_pointers() {
        let bytes = [
            0x08, 0x00, 0x0E, 0x00, 0x0B, 0x00, 0x11, 0x00, b'A', b'B', 0x00, b'C', b'D', 0x00,
            b'E', b'F', 0x00, b'G', 0x00,
        ];
        let mut parsed = MsgFile::parse(&bytes).unwrap();
        parsed.blocks[1].bytes.extend_from_slice(b"1234");
        let rebuilt = parsed.rebuild().unwrap();
        assert_eq!(&rebuilt[2..8], &[0x12, 0x00, 0x0B, 0x00, 0x15, 0x00]);
        let reparsed = MsgFile::parse(&rebuilt).unwrap();
        assert_eq!(reparsed.pointer_block_order, vec![2, 1, 3]);
        assert_eq!(reparsed.blocks[1].bytes, b"CD1234");
    }

    #[test]
    fn recognizes_confirmed_unspaced_name_without_guessing_narration() {
        let (dialogue, _, errors) = SHIFT_JIS.encode("いつみ「はい。」");
        assert!(!errors);
        let segment = parse_display_segment(0, 0, 0..dialogue.len(), &dialogue).unwrap();
        assert_eq!(segment.name.as_deref(), Some("いつみ"));
        assert_eq!(segment.scr_msg, "「はい。」");

        let (narration, _, errors) = SHIFT_JIS.encode("妖華、「......。」");
        assert!(!errors);
        let segment = parse_display_segment(0, 0, 0..narration.len(), &narration).unwrap();
        assert_eq!(segment.name, None);
        assert_eq!(segment.scr_msg, "妖華、「......。」");
    }

    #[test]
    fn editable_controls_can_be_deleted_added_replaced_and_reordered() {
        assert_eq!(
            render_editable_bytes(&[b'A', 0x13, b'B']).unwrap(),
            "A[13]B"
        );
        assert_eq!(decode_editable_text("AB").unwrap(), b"AB");
        assert_eq!(decode_editable_text("A[FC]B").unwrap(), [b'A', 0xFC, b'B']);
        assert_eq!(
            decode_editable_text("[13]A[FC]B").unwrap(),
            [0x13, b'A', 0xFC, b'B']
        );
        assert_eq!(decode_editable_text("[[13]").unwrap(), b"[13]");
        assert!(decode_editable_text("A[0E]B").is_err());
        assert!(decode_editable_text("A[0G]B").is_err());
    }
}
