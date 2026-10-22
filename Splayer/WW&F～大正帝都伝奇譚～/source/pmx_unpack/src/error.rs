use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub type Result<T> = std::result::Result<T, PmxError>;

#[derive(Debug)]
pub struct PmxError {
    message: String,
}

impl PmxError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn context(context: impl Display, error: impl Display) -> Self {
        Self::new(format!("{context}: {error}"))
    }
}

impl Display for PmxError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for PmxError {}
