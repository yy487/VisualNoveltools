use katahane_windows_tool::{InjectAndPack, UnpackAndExtract};
use std::process::ExitCode;
use vn_cli::Panel;

fn run() -> vn_cli::Result<()> {
    Panel::new(
        std::env::current_exe()?.into_os_string(),
        "カタハネ Windows 工具",
        env!("CARGO_PKG_VERSION"),
        vec![Box::new(UnpackAndExtract), Box::new(InjectAndPack)],
    )?
    .run_env()
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[失败] {error}");
            ExitCode::FAILURE
        }
    }
}
