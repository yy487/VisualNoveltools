use encoding_rs::{Encoding, GBK, SHIFT_JIS};

use crate::ToolResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    Cp932,
    Gbk,
}

impl TextEncoding {
    pub fn parse(value: &str) -> ToolResult<Self> {
        match value.to_ascii_lowercase().as_str() {
            "cp932" | "shift-jis" | "shift_jis" | "sjis" => Ok(Self::Cp932),
            "gbk" | "cp936" | "gb2312" => Ok(Self::Gbk),
            _ => Err(format!(
                "unsupported encoding {value:?}; expected cp932 or gbk"
            )),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Cp932 => "cp932",
            Self::Gbk => "gbk",
        }
    }

    fn codec(self) -> &'static Encoding {
        match self {
            Self::Cp932 => SHIFT_JIS,
            Self::Gbk => GBK,
        }
    }

    pub fn decode(self, bytes: &[u8], context: &str) -> ToolResult<String> {
        self.codec()
            .decode_without_bom_handling_and_without_replacement(bytes)
            .map(|value| value.into_owned())
            .ok_or_else(|| {
                format!(
                    "{context}: byte sequence is not valid {} ({} bytes)",
                    self.label(),
                    bytes.len()
                )
            })
    }

    pub fn encode(self, text: &str, context: &str) -> ToolResult<Vec<u8>> {
        if text.contains('\0') {
            return Err(format!("{context}: NUL is not allowed"));
        }
        let (encoded, _, had_errors) = self.codec().encode(text);
        if had_errors {
            let bad = unencodable_chars(self.codec(), text);
            return Err(format!(
                "{context}: characters cannot be encoded as {}: {}",
                self.label(),
                bad.join(" ")
            ));
        }
        Ok(encoded.into_owned())
    }
}

fn unencodable_chars(codec: &'static Encoding, text: &str) -> Vec<String> {
    let mut bad = Vec::new();
    for ch in text.chars() {
        let value = ch.to_string();
        let (_, _, had_errors) = codec.encode(&value);
        if had_errors {
            bad.push(format!("{ch}(U+{:04X})", ch as u32));
        }
    }
    bad.sort();
    bad.dedup();
    bad
}
