pub mod catalog;
pub mod text;
pub mod workflow;

pub type Result<T> = std::result::Result<T, String>;

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
