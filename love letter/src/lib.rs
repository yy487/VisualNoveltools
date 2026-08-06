//! Structure-aware text extraction and injection for Love Letter `.o` scripts.

mod format;
mod text;

pub use format::{parse_obj, Entry, ParsedObj, StringReference, TextUse};
pub use text::{
    extract_entries, inject_entries, ExtractReport, InjectOptions, InjectReport, TextEntry,
};

use std::fmt;

pub type Result<T> = std::result::Result<T, ToolError>;

#[derive(Debug)]
pub enum ToolError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Format(String),
    Text(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Json(err) => write!(f, "JSON error: {err}"),
            Self::Format(msg) => write!(f, "format error: {msg}"),
            Self::Text(msg) => write!(f, "text error: {msg}"),
        }
    }
}

impl std::error::Error for ToolError {}

impl From<std::io::Error> for ToolError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for ToolError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
