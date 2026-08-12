pub mod encoding;
pub mod hash;
pub mod io_util;
pub mod json_model;
pub mod markup;
pub mod ops;
pub mod scenario;
pub mod special;

pub type ToolResult<T> = Result<T, String>;
