#![forbid(unsafe_code)]

pub mod analysis;
pub mod hdi_pack;
pub mod lzss;
pub mod output;
pub mod text;
pub mod workflow;

use sha2::{Digest, Sha256};

pub type Result<T> = std::result::Result<T, String>;

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

pub fn json_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}
