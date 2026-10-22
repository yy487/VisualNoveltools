use std::process::ExitCode;
use suishou_no_dragon_tool::workflow::Action;
fn main() -> ExitCode {
    let result = (|| {
        vn_cli::Panel::new(
            std::env::current_exe()?.into_os_string(),
            "水晶之龙 PRG 扩容与文本闭环",
            env!("CARGO_PKG_VERSION"),
            vec![
                Box::new(Action::Export),
                Box::new(Action::Import),
                Box::new(Action::Inspect),
                Box::new(Action::Convert),
                Box::new(Action::Repair),
            ],
        )?
        .run_env()
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
