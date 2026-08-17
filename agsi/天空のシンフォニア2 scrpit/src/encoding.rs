use encoding_rs::SHIFT_JIS;

use crate::bundle::BundleError;

fn pua_to_bytes(codepoint: u32) -> Option<[u8; 2]> {
    let index = codepoint.checked_sub(0xe000)?;
    if index >= 1880 {
        return None;
    }
    let lead = 0xf0 + (index / 188) as u8;
    let trail_index = index % 188;
    let trail = if trail_index < 63 {
        0x40 + trail_index as u8
    } else {
        0x80 + (trail_index - 63) as u8
    };
    Some([lead, trail])
}

pub fn encode_cp932(text: &str, context: &str) -> Result<Vec<u8>, BundleError> {
    if text.contains('\0') {
        return Err(format!("{} 含 NUL，不能写入 CSTR", context));
    }
    let mut bytes = Vec::new();
    for character in text.chars() {
        if let Some(pair) = pua_to_bytes(character as u32) {
            bytes.extend_from_slice(&pair);
            continue;
        }
        let character_text = character.to_string();
        let (encoded, _, had_errors) = SHIFT_JIS.encode(&character_text);
        if had_errors {
            return Err(format!(
                "{} 含 CP932 不可编码字符 U+{:04X}",
                context, character as u32
            ));
        }
        bytes.extend_from_slice(&encoded);
    }
    let (decoded, _, decode_errors) = SHIFT_JIS.decode(&bytes);
    if decode_errors || decoded != text {
        return Err(format!("{} CP932 编码回环失败", context));
    }
    Ok(bytes)
}

pub fn decode_cp932(bytes: &[u8], context: &str) -> Result<String, BundleError> {
    let (decoded, _, had_errors) = SHIFT_JIS.decode(bytes);
    if had_errors {
        return Err(format!("{} 不是严格 CP932/Shift-JIS 文本", context));
    }
    let text = decoded.into_owned();
    let encoded = encode_cp932(&text, context)?;
    if encoded != bytes {
        return Err(format!("{} CP932 重编码回环失败", context));
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::{decode_cp932, encode_cp932};

    #[test]
    fn round_trips_standard_text_and_newlines() {
        let text = "【ノエル】\n本文テスト～";
        let encoded = encode_cp932(text, "test").unwrap();
        assert_eq!(decode_cp932(&encoded, "test").unwrap(), text);
    }

    #[test]
    fn round_trips_the_complete_windows_31j_user_area() {
        let text = (0xe000..=0xe757)
            .map(char::from_u32)
            .collect::<Option<String>>()
            .unwrap();
        let encoded = encode_cp932(&text, "pua").unwrap();
        assert_eq!(encoded.len(), 1880 * 2);
        assert_eq!(&encoded[..2], &[0xf0, 0x40]);
        assert_eq!(&encoded[encoded.len() - 2..], &[0xf9, 0xfc]);
        assert_eq!(decode_cp932(&encoded, "pua").unwrap(), text);
    }

    #[test]
    fn rejects_nul_and_unencodable_characters() {
        assert!(encode_cp932("a\0b", "nul").unwrap_err().contains("NUL"));
        assert!(encode_cp932("😀", "emoji").unwrap_err().contains("U+1F600"));
    }
}
