use std::env;
use std::fs;
use std::path::PathBuf;

use qc_keisei_tools::anm;

const HELP: &str = "QC / KEISEI BIZ2 ANM inspector\n\nUsage:\n  qc_anm_info.exe FILE.ANM [FILE2.ANM ...]\n\nReads and validates the BIZ2/LZSS stream without modifying the file.\n";

fn main() {
    let inputs = env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if inputs.is_empty() || inputs.iter().any(|path| path == "-h" || path == "--help") {
        print!("{HELP}");
        std::process::exit(if inputs.is_empty() { 1 } else { 0 });
    }
    for input in inputs {
        let result = fs::read(&input)
            .map_err(|error| format!("{}: {error}", input.display()))
            .and_then(|data| {
                anm::inspect(&data).map_err(|error| format!("{}: {error}", input.display()))
            });
        match result {
            Ok(info) => {
                println!("[anm] file={}", input.display());
                println!("[anm] format=BIZ2");
                println!("[anm] dimensions={}x{}", info.width, info.height);
                println!("[anm] pixel_format=BGR24");
                println!("[anm] frames={}", info.frames);
                println!("[anm] compressed_bytes={}", info.compressed_bytes);
                println!("[anm] decoded_bytes={}", info.decoded_bytes);
            }
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
    }
}
