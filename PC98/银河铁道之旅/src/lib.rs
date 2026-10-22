#![forbid(unsafe_code)]

pub mod crs_text;
pub mod data_disks;
pub mod fat_rebuild;
pub mod font_plan;
pub mod text;
pub mod workflow;

use sha2::{Digest, Sha256};

pub type Result<T> = std::result::Result<T, String>;

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
