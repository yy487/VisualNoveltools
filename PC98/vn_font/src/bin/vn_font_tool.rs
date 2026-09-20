use std::process::ExitCode;

fn main() -> ExitCode {
    let result = (|| {
        vn_cli::Panel::new(
            std::env::current_exe()?.into_os_string(),
            "PC88 / PC98 公共字库工具",
            env!("CARGO_PKG_VERSION"),
            vn_font::panel::operations(),
        )?
        .run_env()
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[失败] {error}");
            ExitCode::FAILURE
        }
    }
}
