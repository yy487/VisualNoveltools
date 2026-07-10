use std::process::ExitCode;

use railsoft_xfl_tool::drag::{self, DragOperation};

fn main() -> ExitCode {
    drag::run(DragOperation::GscExtract)
}
