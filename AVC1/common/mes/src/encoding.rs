use std::borrow::Cow;

use encoding_rs::SHIFT_JIS;

use crate::{ToolResult, error};

pub fn decode_cp932(bytes: &[u8], context: &str) -> ToolResult<String> {
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(bytes);
    if had_errors {
        return Err(error(format!(
            "{context}: invalid or unmappable CP932 byte sequence"
        )));
    }

    let text = decoded.into_owned();
    let encoded = encode_cp932(&text, context)?;
    if encoded != bytes {
        return Err(error(format!(
            "{context}: CP932 decode/encode round-trip mismatch"
        )));
    }
    Ok(text)
}

pub fn encode_cp932(text: &str, context: &str) -> ToolResult<Vec<u8>> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        let characters = unencodable_characters(text);
        return Err(error(format!(
            "{context}: characters cannot be encoded as CP932: {}",
            characters.join(", ")
        )));
    }
    Ok(match encoded {
        Cow::Borrowed(bytes) => bytes.to_vec(),
        Cow::Owned(bytes) => bytes,
    })
}

pub fn cp932_len(text: &str) -> ToolResult<usize> {
    encode_cp932(text, "text span").map(|bytes| bytes.len())
}

fn unencodable_characters(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    for character in text.chars() {
        let value = character.to_string();
        let (_, _, had_errors) = SHIFT_JIS.encode(&value);
        if had_errors {
            let display = character.escape_default().to_string();
            let item = format!("U+{:04X} '{display}'", character as u32);
            if !result.contains(&item) {
                result.push(item);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cp932_round_trip() {
        let source = "【文太郎,S000_A_0001】「おつかれーす」[n]";
        let bytes = encode_cp932(source, "test").unwrap();
        assert_eq!(decode_cp932(&bytes, "test").unwrap(), source);
    }

    #[test]
    fn rejects_unencodable_character() {
        let error = encode_cp932("emoji: \u{1f600}", "test").unwrap_err();
        assert!(error.0.contains("U+1F600"));
    }
}
