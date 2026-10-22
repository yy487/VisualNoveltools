#![forbid(unsafe_code)]

use std::error::Error as StdError;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

/// One translator-facing record, matching the secondary JSON view used by `megas`.
///
/// Source identity and format-specific metadata stay in the game adapter's
/// primary extraction model. This type is only the secondary translation view.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub message: String,
}

#[derive(Debug)]
pub enum Error {
    Json(serde_json::Error),
    EntryCountMismatch {
        expected: usize,
        actual: usize,
    },
    NamePresenceMismatch {
        index: usize,
        source_has_name: bool,
        translation_has_name: bool,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid translation JSON: {error}"),
            Self::EntryCountMismatch { expected, actual } => write!(
                formatter,
                "translation has {actual} entries; expected {expected} in source order"
            ),
            Self::NamePresenceMismatch {
                index,
                source_has_name,
                translation_has_name,
            } => write!(
                formatter,
                "translation entry at index {index} changed name-field presence (source={source_has_name}, translation={translation_has_name})"
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::EntryCountMismatch { .. } | Self::NamePresenceMismatch { .. } => None,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

/// Read the translator-facing JSON array. Unknown entry fields are rejected.
pub fn read_template(bytes: &[u8]) -> Result<Vec<Entry>> {
    serde_json::from_slice(bytes).map_err(Error::from)
}

/// Serialize the translator-facing JSON array in pretty form with a final LF.
pub fn write_template(entries: &[Entry]) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(entries).map_err(Error::from)?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Check that a translation view still matches the current source projection.
///
/// This deliberately checks only entry count and whether `name` is present at
/// each position. Both `name` (when present) and `message` are editable, so the
/// adapter must apply them to its own records after validation. It must also
/// bind the JSON file to its source independently; this template has no file or
/// entry identifiers.
pub fn validate_shape(expected: &[Entry], supplied: &[Entry]) -> Result<()> {
    if expected.len() != supplied.len() {
        return Err(Error::EntryCountMismatch {
            expected: expected.len(),
            actual: supplied.len(),
        });
    }

    for (index, (original, edited)) in expected.iter().zip(supplied).enumerate() {
        if original.name.is_some() != edited.name.is_some() {
            return Err(Error::NamePresenceMismatch {
                index,
                source_has_name: original.name.is_some(),
                translation_has_name: edited.name.is_some(),
            });
        }
    }
    Ok(())
}
