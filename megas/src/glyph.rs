use crate::{ToolError, ToolResult};
use encoding_rs::EUC_JP;
use std::collections::{BTreeSet, HashMap};

pub const GLYPH_COUNT: usize = 8194;
pub const GLYPH_RECORD_SIZE: usize = 16;
pub const TOKEN_PREFIX: [u8; 2] = [0x80, 0x00];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    Glyph(u16),
    Byte(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    pub offset: usize,
    pub len: usize,
    pub kind: UnitKind,
}

#[derive(Debug, Clone)]
pub struct Projection {
    pub text: String,
    pub glyph_indices: Vec<u16>,
    pub controls: Vec<u8>,
    pub unresolved: Vec<u16>,
}

/// Characters needed by the translation resource builder after parsing the
/// extraction-layer markup. `render_targets` are all visible characters whose
/// resolved atlas slots must be redrawn; `unmapped` are characters that have
/// neither a built-in carrier nor an existing glyph.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TranslationScan {
    pub render_targets: Vec<char>,
    pub unmapped: Vec<char>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationGlyphUseKind {
    MappedTarget(char),
    Literal(char),
    Markup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranslationGlyphUse {
    pub index: u16,
    pub kind: TranslationGlyphUseKind,
}

/// The project mapping is compiled into the executable.  It maps a translated
/// Simplified Chinese character to a Japanese carrier glyph already present in
/// the game's atlas.
#[derive(Debug, Clone)]
pub struct GlyphDictionary {
    target_to_carrier: HashMap<char, char>,
    char_to_index: HashMap<char, u16>,
    index_to_char: Vec<Option<char>>,
}

impl GlyphDictionary {
    pub fn built_in() -> ToolResult<Self> {
        let mapping: HashMap<String, String> =
            serde_json::from_str(include_str!("../assets/subs_cn_jp.json")).map_err(|error| {
                ToolError(format!("embedded subs_cn_jp mapping is invalid: {error}"))
            })?;
        Self::from_mapping(mapping)
    }

    pub fn from_mapping(mapping: HashMap<String, String>) -> ToolResult<Self> {
        let mut target_to_carrier = HashMap::with_capacity(mapping.len());
        for (target, carrier) in mapping {
            let mut target_chars = target.chars();
            let target_char = target_chars
                .next()
                .ok_or_else(|| ToolError("mapping contains an empty target key".to_string()))?;
            if target_chars.next().is_some() {
                return Err(ToolError(format!(
                    "mapping target {target:?} is not a single Unicode scalar"
                )));
            }
            let mut carrier_chars = carrier.chars();
            let carrier_char = carrier_chars.next().ok_or_else(|| {
                ToolError(format!("mapping target {target:?} has an empty carrier"))
            })?;
            if carrier_chars.next().is_some() {
                return Err(ToolError(format!(
                    "mapping carrier for {target:?} is not a single Unicode scalar"
                )));
            }
            if target_to_carrier
                .insert(target_char, carrier_char)
                .is_some()
            {
                return Err(ToolError(format!(
                    "mapping contains duplicate target {target_char:?}"
                )));
            }
        }

        let mut index_to_char = vec![None; GLYPH_COUNT];
        for (index, character) in known_low_glyphs() {
            index_to_char[index as usize] = Some(character);
        }
        for (index, character) in index_to_char.iter_mut().enumerate() {
            if character.is_none() {
                *character = index_to_jis_char(index as u16);
            }
        }
        let mut char_to_index = HashMap::new();
        for (index, character) in index_to_char.iter().enumerate() {
            if let Some(character) = character {
                // Duplicate visual slots intentionally keep the first known
                // low/special slot so unchanged punctuation remains stable.
                char_to_index.entry(*character).or_insert(index as u16);
            }
        }
        // Keep the carriers used by the embedded CN->JP table on the atlas
        // slots verified for translation.  Several decorative/style tables
        // contain the same visible character later in the index space.
        for (character, index) in [
            ('・', 0x00E2),
            ('―', 0x00E5),
            ('「', 0x00D8),
            ('」', 0x00D9),
            ('『', 0x00DA),
            ('』', 0x00DB),
        ] {
            if index_to_char[index].is_some_and(|value| value == character) {
                char_to_index.insert(character, index as u16);
            }
        }

        for (target, carrier) in &target_to_carrier {
            if !char_to_index.contains_key(carrier) {
                return Err(ToolError(format!(
                    "embedded mapping carrier {carrier:?} for target {target:?} has no glyph index"
                )));
            }
        }

        Ok(Self {
            target_to_carrier,
            char_to_index,
            index_to_char,
        })
    }

    pub fn target_count(&self) -> usize {
        self.target_to_carrier.len()
    }

    pub fn carrier_for(&self, target: char) -> Option<char> {
        self.target_to_carrier.get(&target).copied()
    }

    pub fn index_for_char(&self, character: char) -> Option<u16> {
        self.char_to_index.get(&character).copied()
    }

    pub fn index_for_translated_char(&self, character: char) -> Option<u16> {
        let carrier = self.carrier_for(character).unwrap_or(character);
        self.index_for_char(carrier)
    }

    pub fn carrier_table(&self) -> &HashMap<char, char> {
        &self.target_to_carrier
    }

    pub fn char_table(&self) -> &HashMap<char, u16> {
        &self.char_to_index
    }

    pub fn char_for_index(&self, index: u16) -> Option<char> {
        self.index_to_char.get(index as usize).copied().flatten()
    }

    /// Project the extraction-layer markup into text intended for a translator.
    /// Glyph markup is resolved through the embedded atlas dictionary. Opaque
    /// control bytes are kept out of the editable view; 0x00 is represented as
    /// a line break and terminator/control bytes are otherwise omitted.
    pub fn project_translation_text(&self, text: &str) -> ToolResult<String> {
        project_translation_text(text, self)
    }

    pub fn encode_text(&self, text: &str) -> ToolResult<Vec<u8>> {
        let mut output = Vec::with_capacity(text.len() * 4);
        let mut missing = Vec::new();
        let mut start = 0;
        while start < text.len() {
            let character = text[start..]
                .chars()
                .next()
                .expect("valid UTF-8 character boundary");
            if character == '<' {
                if let Some((end, index)) = parse_markup_at(text, start, "g:", 4)? {
                    output.extend_from_slice(&TOKEN_PREFIX);
                    output.extend_from_slice(&index.to_be_bytes());
                    start = end;
                    continue;
                }
                if let Some((end, value)) = parse_markup_at(text, start, "b:", 2)? {
                    output.push(value as u8);
                    start = end;
                    continue;
                }
            }
            if let Some(index) = self.index_for_translated_char(character) {
                output.extend_from_slice(&TOKEN_PREFIX);
                output.extend_from_slice(&index.to_be_bytes());
            } else {
                missing.push((start, character));
            }
            start += character.len_utf8();
        }
        if !missing.is_empty() {
            let rendered = missing
                .iter()
                .map(|(_, character)| format!("U+{:04X} {:?}", *character as u32, character))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ToolError(format!(
                "unmapped translation characters: {rendered}"
            )));
        }
        Ok(output)
    }

    pub fn scan_translation_text(&self, text: &str) -> ToolResult<TranslationScan> {
        let mut render_targets = BTreeSet::new();
        let mut unmapped = BTreeSet::new();
        let mut start = 0;
        while start < text.len() {
            let character = text[start..]
                .chars()
                .next()
                .expect("valid UTF-8 character boundary");
            if character == '<' {
                if let Some((end, _)) = parse_markup_at(text, start, "g:", 4)? {
                    start = end;
                    continue;
                }
                if let Some((end, _)) = parse_markup_at(text, start, "b:", 2)? {
                    start = end;
                    continue;
                }
            }
            if character.is_whitespace() {
                // Spaces and line breaks do not have visible outlines to draw.
            } else if self.carrier_for(character).is_some()
                || self.index_for_char(character).is_some()
            {
                render_targets.insert(character);
            } else {
                unmapped.insert(character);
            }
            start += character.len_utf8();
        }
        Ok(TranslationScan {
            render_targets: render_targets.into_iter().collect(),
            unmapped: unmapped.into_iter().collect(),
        })
    }

    pub fn scan_translation_glyph_uses(&self, text: &str) -> ToolResult<Vec<TranslationGlyphUse>> {
        let mut uses = Vec::new();
        let mut start = 0;
        while start < text.len() {
            let character = text[start..]
                .chars()
                .next()
                .expect("valid UTF-8 character boundary");
            if character == '<' {
                if let Some((end, index)) = parse_markup_at(text, start, "g:", 4)? {
                    uses.push(TranslationGlyphUse {
                        index,
                        kind: TranslationGlyphUseKind::Markup,
                    });
                    start = end;
                    continue;
                }
                if let Some((end, _)) = parse_markup_at(text, start, "b:", 2)? {
                    start = end;
                    continue;
                }
            }
            if let Some(carrier) = self.carrier_for(character) {
                let index = self
                    .index_for_char(carrier)
                    .expect("validated embedded carrier mapping");
                uses.push(TranslationGlyphUse {
                    index,
                    kind: TranslationGlyphUseKind::MappedTarget(character),
                });
            } else if let Some(index) = self.index_for_char(character) {
                uses.push(TranslationGlyphUse {
                    index,
                    kind: TranslationGlyphUseKind::Literal(character),
                });
            }
            start += character.len_utf8();
        }
        Ok(uses)
    }
}

/// Scan a token stream without assigning semantic meaning to opaque bytes.
pub fn scan_units(data: &[u8]) -> Vec<Unit> {
    let mut units = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        if data[offset..].starts_with(&TOKEN_PREFIX) && offset + 4 <= data.len() {
            let index = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
            units.push(Unit {
                offset,
                len: 4,
                kind: UnitKind::Glyph(index),
            });
            offset += 4;
        } else {
            units.push(Unit {
                offset,
                len: 1,
                kind: UnitKind::Byte(data[offset]),
            });
            offset += 1;
        }
    }
    units
}

pub fn project_units(units: &[Unit], dictionary: &GlyphDictionary) -> Projection {
    let mut text = String::new();
    let mut glyph_indices = Vec::new();
    let mut controls = Vec::new();
    let mut unresolved = Vec::new();
    for unit in units {
        match unit.kind {
            UnitKind::Glyph(index) => {
                glyph_indices.push(index);
                if let Some(character) = dictionary.char_for_index(index) {
                    // A character can have multiple visual slots.  Keep a
                    // non-canonical slot explicit so extracting and injecting
                    // unchanged text remains byte-exact.
                    if dictionary.index_for_char(character) == Some(index) {
                        text.push(character);
                    } else {
                        text.push_str(&format!("<g:{index:04X}>"));
                    }
                } else {
                    text.push_str(&format!("<g:{index:04X}>"));
                    unresolved.push(index);
                }
            }
            UnitKind::Byte(value) => {
                controls.push(value);
                text.push_str(&format!("<b:{value:02X}>"));
            }
        }
    }
    unresolved.sort_unstable();
    unresolved.dedup();
    Projection {
        text,
        glyph_indices,
        controls,
        unresolved,
    }
}

pub fn index_to_char(index: u16) -> Option<char> {
    known_low_glyphs()
        .get(&index)
        .copied()
        .or_else(|| index_to_jis_char(index))
}

pub fn validate_glyph_index(index: u16) -> ToolResult<()> {
    if (index as usize) < GLYPH_COUNT {
        Ok(())
    } else {
        Err(ToolError(format!(
            "glyph index 0x{index:04X} exceeds {GLYPH_COUNT} records"
        )))
    }
}

pub(crate) fn parse_markup_at(
    text: &str,
    start: usize,
    prefix: &str,
    digits: usize,
) -> ToolResult<Option<(usize, u16)>> {
    if start > text.len() {
        return Err(ToolError(format!(
            "markup byte offset {start} exceeds text length {}",
            text.len()
        )));
    }
    if start == text.len() {
        return Ok(None);
    }
    if !text.is_char_boundary(start) {
        return Err(ToolError(format!(
            "markup byte offset {start} is not a UTF-8 character boundary"
        )));
    }
    if text.as_bytes()[start] != b'<' {
        return Ok(None);
    }
    let remainder = &text[start + 1..];
    if !remainder.starts_with(prefix) {
        return Ok(None);
    }
    let Some(close) = remainder.find('>') else {
        return Err(ToolError(format!(
            "unterminated markup at character offset {start}"
        )));
    };
    let digits_text = &remainder[prefix.len()..close];
    if digits_text.len() != digits || !digits_text.bytes().all(|value| value.is_ascii_hexdigit()) {
        return Err(ToolError(format!(
            "invalid <{}...> markup at character offset {start}",
            prefix.trim_end_matches(':')
        )));
    }
    let value = u16::from_str_radix(digits_text, 16).map_err(|error| {
        ToolError(format!(
            "invalid markup at character offset {start}: {error}"
        ))
    })?;
    if prefix == "g:" {
        validate_glyph_index(value)?;
    }
    Ok(Some((start + 1 + close + 1, value)))
}

fn known_low_glyphs() -> HashMap<u16, char> {
    let mut values = HashMap::from([
        (0x0000, ' '),
        // The first low table is the game's single-byte ASCII/punctuation
        // family.  The 0x0080 table is a second visual style of the same set.
        (0x003F, ' '),
        (0x0040, '/'),
        (0x0041, ':'),
        (0x0042, '-'),
        (0x0043, ';'),
        (0x0044, '!'),
        (0x0045, '?'),
        (0x0046, '\''),
        (0x0047, '.'),
        (0x0048, '@'),
        (0x0049, '#'),
        (0x004A, '%'),
        (0x004B, '~'),
        (0x004C, '*'),
        (0x004D, '/'),
        (0x004E, '\\'),
        (0x004F, '('),
        (0x0050, ')'),
        (0x0051, '°'),
        (0x0052, '^'),
        (0x0053, '>'),
        (0x0054, '+'),
        (0x0055, '<'),
        (0x0056, '/'),
        (0x0057, '·'),
        (0x0058, '='),
        (0x0059, '"'),
        (0x005A, '$'),
        (0x005B, '|'),
        (0x005C, ','),
        (0x005D, '['),
        (0x005E, '\\'),
        (0x005F, ']'),
        (0x0060, '&'),
        (0x0061, '{'),
        (0x0062, '|'),
        (0x0063, '}'),
        (0x0064, '□'),
        (0x00BE, '、'),
        (0x00BF, '。'),
        (0x00C0, '，'),
        (0x00C1, '・'),
        (0x00C2, '：'),
        (0x00C3, '；'),
        (0x00C4, '？'),
        (0x00C5, '！'),
        (0x00C6, '―'),
        (0x00C7, '・'),
        (0x00C8, '゛'),
        (0x00C9, '゜'),
        (0x00CA, '“'),
        (0x00CB, '”'),
        (0x00CC, '（'),
        (0x00CD, '）'),
        (0x00CE, '［'),
        (0x00CF, '］'),
        (0x00D0, '［'),
        (0x00D1, '］'),
        (0x00D2, '｛'),
        (0x00D3, '｝'),
        (0x00D4, '〈'),
        (0x00D5, '〉'),
        (0x00D6, '《'),
        (0x00D7, '》'),
        (0x00D8, '「'),
        (0x00D9, '」'),
        (0x00DA, '『'),
        (0x00DB, '』'),
        (0x00DC, '【'),
        (0x00DD, '】'),
        (0x00DE, '〔'),
        (0x00DF, '〕'),
        (0x00E0, '〖'),
        (0x00E1, '〗'),
        (0x00E2, '・'),
        (0x00E3, '…'),
        (0x00E4, '～'),
        (0x00E5, '―'),
        (0x00E6, '♪'),
        (0x00E7, '―'),
        // Small kana are in this dedicated table; the continuous kana table
        // below intentionally contains no small っ/ッ entries.
        (0x00E8, 'あ'),
        (0x00E9, 'い'),
        (0x00EA, 'う'),
        (0x00EB, 'え'),
        (0x00EC, 'お'),
        (0x00ED, 'っ'),
        (0x00EE, 'ゃ'),
        (0x00EF, 'ゅ'),
        (0x00F0, 'ょ'),
        (0x00F1, 'わ'),
        (0x00F2, 'ァ'),
        (0x00F3, 'ィ'),
        (0x00F4, 'ゥ'),
        (0x00F5, 'ェ'),
        (0x00F6, 'ォ'),
        (0x00F7, 'ッ'),
        (0x00F8, 'ャ'),
        (0x00F9, 'ュ'),
        (0x00FA, 'ョ'),
        (0x00FB, 'ヮ'),
        (0x00FC, 'ケ'),
        (0x00FD, 'ヶ'),
        (0x00FE, '①'),
        (0x00FF, '②'),
        (0x0100, '♥'),
        (0x0101, '―'),
        (0x0102, '♯'),
        (0x0112, '%'),
        (0x0113, '―'),
        (0x0116, '/'),
        (0x015F, 'β'),
        (0x0160, 'γ'),
        (0x0172, '＼'),
        (0x0176, '‖'),
        (0x0177, 'α'),
        (0x0179, 'ε'),
        (0x0182, '〃'),
        (0x0184, '〃'),
        (0x0185, '∥'),
        (0x0186, '全'),
        (0x0187, '々'),
        (0x0188, '∅'),
        (0x018B, '+'),
        (0x018C, '―'),
        (0x018E, '×'),
        (0x018F, '÷'),
        (0x0191, '≠'),
        (0x0194, '≤'),
        (0x01A0, '#'),
        (0x01A1, '&'),
        (0x01A4, '§'),
        (0x01A5, '☆'),
        (0x01A7, '○'),
        (0x01A8, '●'),
        (0x01AC, '□'),
        (0x01AE, '△'),
        (0x01B1, '▼'),
        (0x01B2, '※'),
        (0x01B3, '┬'),
        (0x01B6, '↑'),
        (0x01B7, '↓'),
        (0x01B8, '≡'),
        (0x01BA, '⊃'),
        (0x01BB, '⊂'),
        (0x01BE, '∪'),
        (0x01C0, '∩'),
        (0x01C4, '⇒'),
        (0x01C5, '⇔'),
        (0x01CE, '≡'),
        (0x01CF, '―'),
        (0x0270, 'ヴ'),
        (0x0297, '♡'),
        (0x2000, '―'),
        (0x2001, '―'),
    ]);
    for index in 0x0001..=0x000A {
        values.insert(index, char::from(b'0' + (index - 1) as u8));
    }
    for (index, character) in "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().enumerate() {
        values.insert(0x000B + index as u16, character);
    }
    for (index, character) in "abcdefghijklmnopqrstuvwxyz".chars().enumerate() {
        values.insert(0x0025 + index as u16, character);
    }
    for index in 0x0080..=0x00BD {
        let source = index - 0x007F;
        let character = values.get(&source).copied().unwrap_or('□');
        values.insert(index, character);
    }
    // The atlas reserves a run of raised empty/button cells. They are visible
    // glyphs, so project them as a square instead of leaking an opaque token.
    for index in 0x0064..=0x007E {
        values.insert(index, '□');
    }
    for index in 0x011D..=0x015E {
        values.insert(index, '①');
    }
    for index in 0x029B..=0x02BE {
        values.insert(index, '□');
    }
    let hiragana = "あいうえおかがきぎくぐけげこごさざしじすずせぜそぞただちぢつづてでとどなにぬねのはばぱひびぴふぶぷへべぺほぼぽまみむめもやゆよらりるれろわゐゑをん";
    let katakana = "アイウエオカガキギクグケゲコゴサザシジスズセゼソゾタダチヂツヅテデトドナニヌネノハバパヒビピフブプヘベペホボポマミムメモヤユヨラリルレロヮワヰヱヲン";
    for (index, character) in hiragana.chars().enumerate() {
        values.insert(0x01DD + index as u16, character);
    }
    for (index, character) in katakana.chars().enumerate() {
        values.insert(0x0226 + index as u16, character);
    }
    values
}

fn project_translation_text(text: &str, dictionary: &GlyphDictionary) -> ToolResult<String> {
    let mut output = String::new();
    let mut start = 0;
    while start < text.len() {
        let character = text[start..]
            .chars()
            .next()
            .expect("valid UTF-8 character boundary");
        if character == '<' {
            if let Some((end, index)) = parse_markup_at(text, start, "g:", 4)? {
                let projected = dictionary.char_for_index(index).ok_or_else(|| {
                    ToolError(format!(
                        "glyph index 0x{index:04X} has no verified atlas character"
                    ))
                })?;
                output.push(projected);
                start = end;
                continue;
            }
            if let Some((end, value)) = parse_markup_at(text, start, "b:", 2)? {
                if value == 0 {
                    output.push('\n');
                }
                start = end;
                continue;
            }
        }
        output.push(character);
        start += character.len_utf8();
    }
    Ok(output)
}

fn index_to_jis_char(index: u16) -> Option<char> {
    let index = index as usize;
    if index < 0x500 {
        return None;
    }
    let shifted = index.checked_add(2)?;
    let row = shifted / 94 + 0x21;
    let cell = shifted % 94 + 0x21;
    if !(0x21..=0x7E).contains(&row) || !(0x21..=0x7E).contains(&cell) {
        return None;
    }
    let bytes = [row as u8 | 0x80, cell as u8 | 0x80];
    let (decoded, _, had_errors) = EUC_JP.decode(&bytes);
    if had_errors {
        return None;
    }
    let mut chars = decoded.chars();
    let character = chars.next()?;
    if chars.next().is_some() || !(0x3400..=0x9FFF).contains(&(character as u32)) {
        return None;
    }
    Some(character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_projection_matches_confirmed_sample() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let units = scan_units(&hex_bytes(
            "800000d8 800000e3 800000e3 80000820 80000218 8000095d 80000de4 800001f7 800000ed 800001f6 80000203 800000d9",
        ));
        assert_eq!(
            project_units(&units, &dictionary).text,
            "「……君も災難だったね」"
        );
    }

    #[test]
    fn translation_uses_embedded_carrier_map() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let encoded = dictionary.encode_text("这是一句中文。<g:0218>").unwrap();
        assert_eq!(&encoded[..2], &TOKEN_PREFIX);
        let index = dictionary.index_for_translated_char('这').unwrap();
        assert!(encoded
            .windows(4)
            .any(|window| window == [0x80, 0, (index >> 8) as u8, index as u8]));
    }

    #[test]
    fn data_ma_translation_extras_have_unique_carrier_slots() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let indices = "呣焗０１３５７"
            .chars()
            .map(|character| dictionary.index_for_translated_char(character).unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(indices.len(), 7);
    }

    #[test]
    fn malformed_markup_is_rejected() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        assert!(dictionary.encode_text("<g:123>").is_err());
        assert!(dictionary.encode_text("<b:GG>").is_err());
    }

    #[test]
    fn markup_parser_ignores_multibyte_text_at_a_valid_boundary() {
        assert_eq!(parse_markup_at("……", 0, "b:", 2).unwrap(), None);
        assert!(parse_markup_at("……", 1, "b:", 2).is_err());
    }

    #[test]
    fn duplicate_visual_slots_stay_explicit_for_round_trip() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let source = [0x80, 0x00, 0x01, 0x13];
        let projection = project_units(&scan_units(&source), &dictionary);
        assert_eq!(projection.text, "<g:0113>");
        assert_eq!(dictionary.encode_text(&projection.text).unwrap(), source);
    }

    #[test]
    fn atlas_projection_resolves_duplicate_and_special_slots() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let text = "<g:0113><g:2001><g:0221><g:01FA><g:01FC><g:01FE><g:0226><g:0243><g:0270>";
        assert_eq!(
            dictionary.project_translation_text(text).unwrap(),
            "――わつてとアツヴ"
        );
        assert!(!dictionary
            .project_translation_text(text)
            .unwrap()
            .contains("<g:"));
    }

    #[test]
    fn translation_scan_includes_mapped_and_literal_visible_glyphs() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let scan = dictionary
            .scan_translation_text("这是 测试。<g:0218><b:03>\n")
            .unwrap();
        assert!(scan.unmapped.is_empty());
        assert!(scan.render_targets.contains(&'这'));
        assert!(scan.render_targets.contains(&'测'));
        assert!(scan.render_targets.contains(&'是'));
        assert!(scan.render_targets.contains(&'。'));
        assert!(!scan.render_targets.contains(&' '));
        assert!(!scan.render_targets.contains(&'\n'));
    }

    fn hex_bytes(value: &str) -> Vec<u8> {
        value
            .split_whitespace()
            .flat_map(|part| {
                part.as_bytes()
                    .chunks(2)
                    .map(|chunk| {
                        u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap()
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}
