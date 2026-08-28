use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result, bail, ensure};
use encoding_rs::SHIFT_JIS;
use serde::Deserialize;

const BUILTIN_SUBSTITUTIONS_JSON: &str = include_str!("../subs_cn_jp.json");

#[derive(Debug, Default, Clone)]
pub struct CharacterMap {
    encode: BTreeMap<char, [u8; 2]>,
    decode: BTreeMap<[u8; 2], char>,
}

#[derive(Debug, Default, Clone)]
pub struct CharacterSubstitutions {
    replacements: BTreeMap<char, char>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MapFile {
    Direct(BTreeMap<String, String>),
    Wrapped {
        characters: BTreeMap<String, String>,
    },
}

impl CharacterSubstitutions {
    pub fn built_in() -> Result<Self> {
        let source: BTreeMap<String, String> = serde_json::from_str(BUILTIN_SUBSTITUTIONS_JSON)
            .context("invalid built-in subs_cn_jp.json")?;
        let mut replacements = BTreeMap::new();
        for (key, value) in source {
            let from = one_scalar(&key)
                .with_context(|| format!("invalid built-in substitution key {key:?}"))?;
            let to = one_scalar(&value)
                .with_context(|| format!("invalid built-in substitution value for {key:?}"))?;
            CharacterMap::default()
                .encode_char(to)
                .with_context(|| format!("invalid built-in substitution target {to:?}"))?;
            ensure!(
                replacements.insert(from, to).is_none(),
                "duplicate built-in substitution for {from:?}"
            );
        }
        Ok(Self { replacements })
    }

    pub fn apply(&self, text: &str) -> (String, usize) {
        let mut count = 0;
        let converted = text
            .chars()
            .map(|ch| {
                self.replacements.get(&ch).copied().map_or(ch, |mapped| {
                    count += 1;
                    mapped
                })
            })
            .collect();
        (converted, count)
    }
}

impl CharacterMap {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read character map {}", path.display()))?;
        let parsed: MapFile = serde_json::from_slice(&bytes)
            .with_context(|| format!("invalid UTF-8 JSON character map {}", path.display()))?;
        let source = match parsed {
            MapFile::Direct(map) => map,
            MapFile::Wrapped { characters } => characters,
        };
        let mut result = Self::default();
        for (key, value) in source {
            let mut chars = key.chars();
            let ch = chars
                .next()
                .with_context(|| format!("empty character-map key in {}", path.display()))?;
            ensure!(
                chars.next().is_none(),
                "character-map key {key:?} is not one Unicode scalar"
            );
            let code =
                parse_hex_pair(&value).with_context(|| format!("invalid mapping for {key:?}"))?;
            ensure!(
                result.encode.insert(ch, code).is_none(),
                "duplicate mapping for {ch:?}"
            );
            if let Some(previous) = result.decode.insert(code, ch) {
                bail!(
                    "character-map code {:02X}{:02X} maps to both {previous:?} and {ch:?}",
                    code[0],
                    code[1]
                );
            }
        }
        Ok(result)
    }

    fn decode_pair(&self, pair: [u8; 2]) -> Result<char> {
        if let Some(ch) = self.decode.get(&pair) {
            return Ok(*ch);
        }
        let (decoded, _, had_errors) = SHIFT_JIS.decode(&pair);
        ensure!(
            !had_errors && decoded.chars().count() == 1,
            "invalid CP932 code {:02X}{:02X}",
            pair[0],
            pair[1]
        );
        let ch = decoded.chars().next().expect("checked one character");
        let (encoded, _, encode_errors) = SHIFT_JIS.encode(&decoded);
        ensure!(
            !encode_errors && encoded.as_ref() == pair,
            "CP932 code {:02X}{:02X} is not byte-exact",
            pair[0],
            pair[1]
        );
        Ok(ch)
    }

    fn encode_char(&self, ch: char) -> Result<[u8; 2]> {
        if let Some(pair) = self.encode.get(&ch) {
            return Ok(*pair);
        }
        let text = ch.to_string();
        let (encoded, _, had_errors) = SHIFT_JIS.encode(&text);
        ensure!(
            !had_errors,
            "character {ch:?} is not encodable; add it to the character map"
        );
        ensure!(
            encoded.len() == 2,
            "character {ch:?} encodes to {} byte(s), but SDT text requires a double-byte glyph",
            encoded.len()
        );
        Ok([encoded[0], encoded[1]])
    }
}

pub fn decode_mixed(raw: &[u8], map: &CharacterMap) -> Result<String> {
    let mut output = String::new();
    let mut row = 0x24_u8;
    let mut cursor = 0;
    while cursor < raw.len() {
        match raw[cursor] {
            0x20 => {
                let command = *raw.get(cursor + 1).context("truncated 0x20 text control")?;
                match command {
                    b'c' | b'C' | b'w' | b'W' | b'r' | b'R' => {
                        let value = *raw
                            .get(cursor + 2)
                            .context("truncated parameterized text control")?;
                        if value.is_ascii_digit() {
                            output.push('<');
                            output.push(command as char);
                            output.push(value as char);
                            output.push('>');
                        } else {
                            output.push_str(&format!("<{}#{value:02X}>", command as char));
                        }
                        cursor += 3;
                    }
                    b'$' | b'p' | b'P' => {
                        output.push('<');
                        output.push(command as char);
                        output.push('>');
                        cursor += 2;
                    }
                    _ => bail!("unknown 0x20 text control 0x{command:02X}"),
                }
            }
            0x7E => {
                row = 0x24;
                cursor += 1;
            }
            0x7F => {
                row = 0x25;
                cursor += 1;
            }
            0x7D => {
                let value = *raw.get(cursor + 1).context("truncated gaiji control")?;
                ensure!(value.is_ascii_digit(), "invalid gaiji slot 0x{value:02X}");
                output.push_str(&format!("<g{}>", value as char));
                cursor += 2;
            }
            cell @ 0x21..=0x7C => {
                output.push(map.decode_pair(jis_to_sjis(row, cell)?)?);
                cursor += 1;
            }
            first => {
                let second = *raw
                    .get(cursor + 1)
                    .context("truncated encrypted CP932 glyph")?;
                output.push(map.decode_pair([first ^ 0x0A, second ^ 0x0A])?);
                cursor += 2;
            }
        }
    }
    Ok(output)
}

pub fn encode_mixed(text: &str, map: &CharacterMap) -> Result<Vec<u8>> {
    ensure!(
        !text.contains(['\0', '\r', '\n']),
        "NUL and real CR/LF are not valid SDT text"
    );
    let chars: Vec<char> = text.chars().collect();
    let mut output = Vec::with_capacity(chars.len() * 2);
    let mut cursor = 0;
    while cursor < chars.len() {
        if chars[cursor] == '<' {
            let end = chars[cursor + 1..]
                .iter()
                .position(|ch| *ch == '>')
                .map(|index| cursor + 1 + index)
                .context("unterminated '<' control")?;
            let control: String = chars[cursor + 1..end].iter().collect();
            encode_control(&control, &mut output)?;
            cursor = end + 1;
            continue;
        }
        let pair = map.encode_char(chars[cursor])?;
        output.extend([pair[0] ^ 0x0A, pair[1] ^ 0x0A]);
        cursor += 1;
    }
    Ok(output)
}

pub fn decode_choice(raw: &[u8], map: &CharacterMap) -> Result<String> {
    ensure!(
        raw.len().is_multiple_of(2),
        "choice payload has an odd byte length"
    );
    let mut output = String::new();
    for pair in raw.chunks_exact(2) {
        output.push(map.decode_pair([pair[0] ^ 0x0A, pair[1] ^ 0x0A])?);
    }
    Ok(output)
}

pub fn encode_choice(text: &str, map: &CharacterMap) -> Result<Vec<u8>> {
    ensure!(
        !text.contains(['\0', '\r', '\n', '<', '>']),
        "choice contains a forbidden control or newline"
    );
    let mut output = Vec::with_capacity(text.len() * 2);
    for ch in text.chars() {
        let pair = map.encode_char(ch)?;
        output.extend([pair[0] ^ 0x0A, pair[1] ^ 0x0A]);
    }
    ensure!(
        output.len() <= 40,
        "choice encodes to {} bytes; runtime limit is 40",
        output.len()
    );
    Ok(output)
}

fn encode_control(control: &str, output: &mut Vec<u8>) -> Result<()> {
    let bytes = control.as_bytes();
    match bytes {
        [command @ (b'c' | b'C' | b'w' | b'W' | b'r' | b'R'), value] if value.is_ascii_digit() => {
            output.extend([0x20, *command, *value]);
        }
        [
            command @ (b'c' | b'C' | b'w' | b'W' | b'r' | b'R'),
            b'#',
            high,
            low,
        ] if high.is_ascii_hexdigit() && low.is_ascii_hexdigit() => {
            let value = (hex_nibble(*high) << 4) | hex_nibble(*low);
            ensure!(value != 0, "NUL is not a valid raw control parameter");
            output.extend([0x20, *command, value]);
        }
        [b'p'] | [b'P'] | [b'$'] => output.extend([0x20, bytes[0]]),
        [b'g', value] if value.is_ascii_digit() => output.extend([0x7D, *value]),
        _ => bail!("unknown or malformed text control <{control}>"),
    }
    Ok(())
}

fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => unreachable!("caller validates hexadecimal digits"),
    }
}

fn jis_to_sjis(row: u8, cell: u8) -> Result<[u8; 2]> {
    ensure!(
        (0x21..=0x7E).contains(&row) && (0x21..=0x7E).contains(&cell),
        "invalid JIS cell"
    );
    let lead = ((row + 1) >> 1) + if row < 0x5F { 0x70 } else { 0xB0 };
    let trail = if row & 1 != 0 {
        cell + if cell < 0x60 { 0x1F } else { 0x20 }
    } else {
        cell + 0x7E
    };
    Ok([lead, trail])
}

fn parse_hex_pair(value: &str) -> Result<[u8; 2]> {
    let compact: String = value
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect();
    ensure!(
        compact.len() == 4,
        "expected exactly four hexadecimal digits"
    );
    Ok([
        u8::from_str_radix(&compact[0..2], 16)?,
        u8::from_str_radix(&compact[2..4], 16)?,
    ])
}

fn one_scalar(value: &str) -> Result<char> {
    let mut chars = value.chars();
    let ch = chars
        .next()
        .context("expected one Unicode scalar, got empty text")?;
    ensure!(
        chars.next().is_none(),
        "expected one Unicode scalar, got {value:?}"
    );
    Ok(ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_round_trip_preserves_controls() {
        let map = CharacterMap::default();
        let text = "な、なによ！<w1>じっ☆<g0><r2><p>";
        let encoded = encode_mixed(text, &map).unwrap();
        assert_eq!(decode_mixed(&encoded, &map).unwrap(), text);
    }

    #[test]
    fn compressed_kana_decodes() {
        let map = CharacterMap::default();
        assert_eq!(
            decode_mixed(&[0x7F, 0x6A, 0x23, 0x4A], &map).unwrap(),
            "リィナ"
        );
    }

    #[test]
    fn non_digit_control_parameter_is_lossless() {
        let map = CharacterMap::default();
        let raw = [0x20, b'w', 0x2F];
        let decoded = decode_mixed(&raw, &map).unwrap();
        assert_eq!(decoded, "<w#2F>");
        assert_eq!(encode_mixed(&decoded, &map).unwrap(), raw);
    }

    #[test]
    fn rejects_ascii_single_byte_glyphs() {
        let error = encode_mixed("A", &CharacterMap::default()).unwrap_err();
        assert!(error.to_string().contains("requires a double-byte glyph"));
    }

    #[test]
    fn rejects_malformed_controls_and_real_newlines() {
        let map = CharacterMap::default();
        assert!(encode_mixed("本文<w12>", &map).is_err());
        assert!(encode_mixed("本文<unknown>", &map).is_err());
        assert!(encode_mixed("本文\n続き", &map).is_err());
        assert!(encode_mixed("本文\0続き", &map).is_err());
        assert!(encode_mixed("本文<w#00>", &map).is_err());
    }

    #[test]
    fn choice_runtime_limit_is_enforced() {
        let error = encode_choice(
            "あいうえおかきくけこさしすせそたちつてとな",
            &CharacterMap::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("runtime limit is 40"));
    }

    #[test]
    fn custom_character_map_round_trips() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("mapping.json");
        fs::write(&path, r#"{"characters":{"你":"FA40"}}"#.as_bytes()).unwrap();
        let map = CharacterMap::load(Some(&path)).unwrap();
        let encoded = encode_mixed("你", &map).unwrap();
        assert_eq!(encoded, [0xF0, 0x4A]);
        assert_eq!(decode_mixed(&encoded, &map).unwrap(), "你");
    }

    #[test]
    fn built_in_substitutions_convert_translation_characters() {
        let substitutions = CharacterSubstitutions::built_in().unwrap();
        let (converted, count) = substitutions.apply("你说·—");
        assert_eq!(converted, "凜説・―");
        assert_eq!(count, 4);
        assert!(encode_mixed(&converted, &CharacterMap::default()).is_ok());
    }
}
