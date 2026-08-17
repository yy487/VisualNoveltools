pub mod bundle;
pub mod cli;
pub mod encoding;
pub mod extract;
pub mod inject;
pub mod json_model;
pub mod workspace;

pub use bundle::{Bundle, BundleError, CStrEntry, SourceInfo};
pub use extract::{extract_workspace, ExtractOptions, ExtractReport};
pub use inject::{inject_workspace, InjectOptions, InjectReport};
