use std::process::ExitCode;

fn main() -> ExitCode {
    match sekiryuuou_tool::workflow::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[失败] {error}");
            ExitCode::FAILURE
        }
    }
}
