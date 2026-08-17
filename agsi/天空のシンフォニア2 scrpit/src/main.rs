fn main() {
    if let Err(error) = sinfonia_script_tool::cli::run(std::env::args().skip(1).collect()) {
        eprintln!("错误: {}", error);
        std::process::exit(1);
    }
}
