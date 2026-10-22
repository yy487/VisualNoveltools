use executive_road_pc88_tool::{
    ExportTextOperation, ExtractOperation, ImportTextOperation, UnpackOperation,
};
use std::process::ExitCode;
use vn_cli::Panel;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> vn_cli::Result<()> {
    let mut panel = Panel::new(
        std::env::current_exe()?.into_os_string(),
        "エグゼクティブへの道 PC-88 工具",
        env!("CARGO_PKG_VERSION"),
        vec![
            Box::new(UnpackOperation),
            Box::new(ExtractOperation),
            Box::new(ExportTextOperation),
            Box::new(ImportTextOperation),
        ],
    )?;
    panel.run_env()
}
