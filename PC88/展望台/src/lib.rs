#![forbid(unsafe_code)]

pub mod disk;
pub mod disk_rebuild;
pub mod font;
pub mod operations;
pub mod output;
pub mod program;
pub mod text;

pub type Result<T> = std::result::Result<T, String>;

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(bytes);
    format!("{:X}", hash.finalize())
}
