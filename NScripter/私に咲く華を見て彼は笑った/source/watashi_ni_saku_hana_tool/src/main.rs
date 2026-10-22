use std::process::ExitCode;
use vn_cli::Panel;
use watashi_saku_hana_tool::{ExportText, ImportText, Pack, PackArchive, Unpack, UnpackArchive};

fn run() -> vn_cli::Result<()> {
    Panel::new(
        std::env::current_exe()?.into_os_string(),
        "私に咲く華を見て彼は笑った 工具",
        env!("CARGO_PKG_VERSION"),
        vec![
            Box::new(Unpack),
            Box::new(UnpackArchive),
            Box::new(PackArchive),
            Box::new(Pack),
            Box::new(ExportText),
            Box::new(ImportText),
        ],
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
