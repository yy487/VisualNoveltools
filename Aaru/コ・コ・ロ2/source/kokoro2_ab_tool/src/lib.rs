pub mod ab;
pub mod ab_text;
pub mod ab_workflow;

use std::fmt;

pub type ToolResult<T> = Result<T, ToolError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolError(pub String);

impl fmt::Display for ToolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ToolError {}

pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}
