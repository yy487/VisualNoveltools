use crate::{hex, Result};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextEntry {
    pub _file: String,
    pub _index: usize,
    pub _offset: usize,
    pub _source_offset: usize,
    pub _byte_length: usize,
    pub _kind: String,
    pub _raw_hex: String,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Slot {
    pub index: usize,
    pub offset: usize,
    pub byte_length: usize,
    pub kind: String,
    pub raw_hex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_message_id: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Translation {
    pub schema: String,
    pub source: String,
    pub source_offset: usize,
    pub source_sha256: String,
    pub encoding: String,
    pub entries: Vec<TextEntry>,
    pub slots: Vec<Slot>,
}

/// JIS row/cell bytes are stored in display order, not Shift-JIS byte order.
/// 21 77 is a renderer line break, not a displayed fullwidth at-sign.
pub fn decode(raw: &[u8]) -> Result<String> {
    if !raw.len().is_multiple_of(2) {
        return Err("odd JIS byte length".into());
    }
    let mut text = String::new();
    for (i, pair) in raw.chunks_exact(2).enumerate() {
        let (row, cell) = (pair[0], pair[1]);
        if pair == [0x21, 0x77] {
            text.push('\n');
            continue;
        }
        if !(0x21..=0x7e).contains(&row) || !(0x21..=0x7e).contains(&cell) {
            return Err(format!("invalid JIS at +{:#x}: {}", i * 2, hex(pair)));
        }
        let mut lead = ((row - 0x21) >> 1) + 0x81;
        if lead > 0x9f {
            lead += 0x40;
        }
        let trail = cell
            + if row & 1 == 0 {
                0x7e
            } else if cell < 0x60 {
                0x1f
            } else {
                0x20
            };
        let sjis = [lead, trail];
        let value = SHIFT_JIS
            .decode_without_bom_handling_and_without_replacement(&sjis)
            .ok_or_else(|| format!("undefined JIS at +{:#x}: {}", i * 2, hex(pair)))?;
        let (encoded, _, failed) = SHIFT_JIS.encode(&value);
        if failed || encoded.as_ref() != sjis {
            return Err(format!("noncanonical CP932 mapping: {}", hex(pair)));
        }
        text.push_str(&value);
    }
    if encode(&text)? != raw {
        return Err("JIS decode/encode roundtrip mismatch".into());
    }
    Ok(text)
}

/// Exact native encoding for extraction checks. Injection uses the shared font plan.
pub fn encode(text: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    for ch in text.chars() {
        if ch == '\n' {
            bytes.extend([0x21, 0x77]);
            continue;
        }
        let s = ch.to_string();
        let (encoded, _, failed) = SHIFT_JIS.encode(&s);
        if failed || encoded.len() != 2 {
            return Err(format!("not a double-byte JIS character: {ch:?}"));
        }
        let (lead, trail) = (encoded[0], encoded[1]);
        let base = if (0x81..=0x9f).contains(&lead) {
            lead - 0x81
        } else if (0xe0..=0xef).contains(&lead) {
            lead - 0xc1
        } else {
            return Err(format!("outside supported JIS plane: {ch:?}"));
        };
        let mut row = base * 2 + 0x21;
        let cell = if trail >= 0x9f {
            row += 1;
            trail - 0x7e
        } else if trail >= 0x80 {
            trail - 0x20
        } else if trail >= 0x40 {
            trail - 0x1f
        } else {
            return Err(format!("invalid CP932 trail: {ch:?}"));
        };
        if !(0x21..=0x7e).contains(&row)
            || !(0x21..=0x7e).contains(&cell)
            || [row, cell] == [0x21, 0x77]
        {
            return Err(format!("reserved or invalid JIS character: {ch:?}"));
        }
        bytes.extend([row, cell]);
    }
    Ok(bytes)
}

/// All slots are retained, including empty entries, fallback and redirects.
/// Slot numbers, rather than matching text, determine identities.
pub fn parse(
    file: &str,
    source: &str,
    source_offset: usize,
    raw: &[u8],
    kind: &str,
    script_controls: bool,
) -> Result<Translation> {
    if raw.is_empty() || !raw.len().is_multiple_of(2) {
        return Err(format!("{file}: empty or odd-sized string table"));
    }
    let mut result = Translation {
        schema: "kohaku-translation-v1".into(),
        source: source.into(),
        source_offset,
        source_sha256: fivec_new::sha256(raw),
        encoding: "JIS X 0208 row-cell / CP932 mapping; 2177 = LF".into(),
        entries: Vec::new(),
        slots: Vec::new(),
    };
    let mut start = 0;
    for (i, pair) in raw.chunks_exact(2).enumerate() {
        if pair != [0, 0] {
            continue;
        }
        let end = i * 2;
        let body = &raw[start..end];
        let index = result.slots.len();
        let mut target = None;
        let entry_kind = if body.is_empty() {
            "empty"
        } else if script_controls && body == [0, 1] {
            if index < 2 {
                return Err(format!("{file}: fallback has no preceding text slot"));
            }
            "previous-message"
        } else if script_controls
            && body.len() == 2
            && body[0] == 0x23
            && (0x41..=0x46).contains(&body[1])
        {
            target = Some(0x82 + usize::from(body[1] - 0x40));
            "redirect"
        } else if file == "KOHAKU.COM/labels" && (170..184).contains(&index) {
            // The fourteen music identifiers are preserved but are not translation entries.
            "resource-label"
        } else {
            kind
        };
        if !matches!(
            entry_kind,
            "empty" | "previous-message" | "redirect" | "resource-label"
        ) {
            let message = decode(body).map_err(|e| format!("{file} slot {index}: {e}"))?;
            result.entries.push(TextEntry {
                _file: file.into(),
                _index: index,
                _offset: start,
                _source_offset: source_offset + start,
                _byte_length: body.len(),
                _kind: entry_kind.into(),
                _raw_hex: hex(body),
                scr_msg: message.clone(),
                message,
            });
        }
        result.slots.push(Slot {
            index,
            offset: start,
            byte_length: body.len(),
            kind: entry_kind.into(),
            raw_hex: hex(body),
            target_message_id: target,
        });
        start = end + 2;
    }
    if start != raw.len() || result.slots[0].kind != "empty" {
        return Err(format!(
            "{file}: missing initial sentinel or final terminator"
        ));
    }
    Ok(result)
}
