pub mod cli;
pub mod encoding;
pub mod script;
pub mod text_json;
pub mod workflow;

use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ToolError(pub String);

impl Display for ToolError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for ToolError {}

impl From<std::io::Error> for ToolError {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<serde_json::Error> for ToolError {
    fn from(error: serde_json::Error) -> Self {
        Self(error.to_string())
    }
}

pub type ToolResult<T> = Result<T, ToolError>;

pub fn error(message: impl Into<String>) -> ToolError {
    ToolError(message.into())
}
