use anyhow::{anyhow, bail, Result};
use encoding_rs::{Encoding, GBK, SHIFT_JIS, UTF_8};

fn normalized_label(label: &str) -> String {
    label.trim().to_ascii_lowercase().replace(['-', '_'], "")
}

pub fn resolve_encoding(label: &str) -> Result<&'static Encoding> {
    let normalized = normalized_label(label);
    let encoding = match normalized.as_str() {
        "cp932" | "windows31j" | "mskanji" | "shiftjis" | "sjis" => SHIFT_JIS,
        "utf8" | "utf8sig" => UTF_8,
        "gbk" | "gb2312" | "gb18030" => GBK,
        _ => Encoding::for_label(label.trim().as_bytes())
            .ok_or_else(|| anyhow!("unsupported text encoding: {label}"))?,
    };
    Ok(encoding)
}

pub fn decode_text(data: &[u8], label: &str) -> Result<String> {
    let encoding = resolve_encoding(label)?;
    encoding
        .decode_without_bom_handling_and_without_replacement(data)
        .map(|text| text.into_owned())
        .ok_or_else(|| anyhow!("invalid {label} byte sequence"))
}

pub fn try_decode_text(data: &[u8], label: &str) -> Result<Option<String>> {
    let encoding = resolve_encoding(label)?;
    Ok(encoding
        .decode_without_bom_handling_and_without_replacement(data)
        .map(|text| text.into_owned()))
}

pub fn encode_text(text: &str, label: &str) -> Result<Vec<u8>> {
    if text.contains('\0') {
        bail!("text contains NUL");
    }
    let encoding = resolve_encoding(label)?;
    let (encoded, _, had_errors) = encoding.encode(text);
    if had_errors {
        let invalid = unencodable_characters(text, encoding);
        bail!(
            "text is not representable as {label}; unencodable characters: {}",
            invalid.join(" ")
        );
    }
    Ok(encoded.into_owned())
}

fn unencodable_characters(text: &str, encoding: &'static Encoding) -> Vec<String> {
    let mut result = Vec::new();
    for ch in text.chars() {
        let (_, _, had_errors) = encoding.encode(&ch.to_string());
        if had_errors {
            let item = format!("{ch}(U+{:04X})", ch as u32);
            if !result.contains(&item) {
                result.push(item);
            }
        }
    }
    result
}

pub fn is_multibyte_lead(byte: u8, label: &str) -> bool {
    match normalized_label(label).as_str() {
        "utf8" | "utf8sig" => byte >= 0xc0,
        "gbk" | "gb2312" | "gb18030" | "cp932" | "windows31j" | "mskanji" | "shiftjis" | "sjis" => {
            byte >= 0x81
        }
        _ => byte >= 0x81,
    }
}

pub fn is_shift_jis(label: &str) -> bool {
    matches!(
        normalized_label(label).as_str(),
        "cp932" | "windows31j" | "mskanji" | "shiftjis" | "sjis"
    )
}

pub fn is_utf8(label: &str) -> bool {
    matches!(normalized_label(label).as_str(), "utf8" | "utf8sig")
}

pub fn utf8_byte_count(lead: u8) -> usize {
    match lead {
        0x00..=0xbf => 1,
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cp932_roundtrip_is_strict() {
        let source = "名前と本文";
        let encoded = encode_text(source, "cp932").unwrap();
        assert_eq!(decode_text(&encoded, "cp932").unwrap(), source);
        assert!(encode_text("简", "cp932").is_err());
    }
}
