fn main() {
    let arguments = std::env::args_os().skip(1).collect();
    if let Err(error) = ianzhong_mes_text::cli::run_inject(arguments) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
