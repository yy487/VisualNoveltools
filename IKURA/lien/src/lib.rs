pub mod archive;
pub mod script;
pub mod text;
pub mod workflow;

use sha2::{Digest, Sha256};
use std::fmt::Write as _;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, BoxError>;

pub fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(message.into().into())
}

pub fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    let mut output = String::with_capacity(64);
    for byte in digest {
        write!(&mut output, "{byte:02X}").expect("writing to String cannot fail");
    }
    output
}

pub fn hex_encode(data: &[u8]) -> String {
    let mut output = String::with_capacity(data.len() * 2);
    for byte in data {
        write!(&mut output, "{byte:02X}").expect("writing to String cannot fail");
    }
    output
}

pub fn hex_decode(value: &str) -> Result<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return fail("hex string has an odd number of digits");
    }
    let mut output = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks_exact(2) {
        let text = std::str::from_utf8(pair)?;
        output.push(u8::from_str_radix(text, 16)?);
    }
    Ok(output)
}
