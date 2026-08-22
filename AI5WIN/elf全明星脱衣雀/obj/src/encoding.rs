use crate::{fail, Result};
use encoding_rs::{Encoding, GBK, SHIFT_JIS};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EncodingRoute {
    Cp932,
    Gbk,
}

impl EncodingRoute {
    pub fn label(self) -> &'static str {
        match self {
            Self::Cp932 => "cp932",
            Self::Gbk => "gbk",
        }
    }

    fn encoding(self) -> &'static Encoding {
        match self {
            Self::Cp932 => SHIFT_JIS,
            Self::Gbk => GBK,
        }
    }
}

impl fmt::Display for EncodingRoute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

impl FromStr for EncodingRoute {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "cp932" | "sjis" | "shift_jis" => Ok(Self::Cp932),
            "gbk" | "cp936" => Ok(Self::Gbk),
            _ => Err(format!(
                "unsupported encoding {value:?}; expected cp932 or gbk"
            )),
        }
    }
}

pub fn decode_text(data: &[u8], route: EncodingRoute) -> Result<String> {
    let (decoded, had_errors) = route.encoding().decode_without_bom_handling(data);
    if had_errors {
        return fail(format!("text is not valid {route}"));
    }
    let text = decoded.into_owned();
    if encode_text(&text, route)? != data {
        return fail(format!("text does not round-trip through {route}"));
    }
    Ok(text)
}

pub fn encode_text(text: &str, route: EncodingRoute) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(text.len() * 2);
    for (character_index, character) in text.chars().enumerate() {
        if matches!(character, '\0' | '\r' | '\n') {
            return fail(format!(
                "forbidden character U+{:04X} at character index {character_index}",
                character as u32
            ));
        }
        let mut buffer = [0u8; 4];
        let one = character.encode_utf8(&mut buffer);
        let (encoded, _, had_errors) = route.encoding().encode(one);
        if had_errors {
            return fail(format!(
                "character {character:?} (U+{:04X}) at character index {character_index} is not encodable as {route}",
                character as u32
            ));
        }
        output.extend_from_slice(encoded.as_ref());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_routes_are_strict() {
        let cp932 = encode_text("高揚感", EncodingRoute::Cp932).unwrap();
        assert_eq!(decode_text(&cp932, EncodingRoute::Cp932).unwrap(), "高揚感");
        let gbk = encode_text("高扬感", EncodingRoute::Gbk).unwrap();
        assert_eq!(decode_text(&gbk, EncodingRoute::Gbk).unwrap(), "高扬感");
    }
}
