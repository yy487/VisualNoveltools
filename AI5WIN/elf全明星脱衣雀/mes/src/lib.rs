pub mod encoding;
pub mod mes;
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
