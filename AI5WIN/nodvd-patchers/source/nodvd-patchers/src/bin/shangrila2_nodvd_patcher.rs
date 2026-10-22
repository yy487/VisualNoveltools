use ai5win_nodvd_patchers::{run_cli, SHANGRILA_2};

fn main() {
    if let Err(error) = run_cli(&SHANGRILA_2) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
