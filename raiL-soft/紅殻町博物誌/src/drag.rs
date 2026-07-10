use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::ExitCode;

use crate::workflow::{
    extract_gsc_default, inject_gsc_default, pack_xfl_default, unpack_xfl_default,
};

#[derive(Debug, Clone, Copy)]
pub enum DragOperation {
    XflUnpack,
    XflPack,
    GscExtract,
    GscInject,
}

#[must_use]
pub fn run(operation: DragOperation) -> ExitCode {
    let mut arguments: Vec<OsString> = env::args_os().skip(1).collect();
    let no_pause = arguments
        .iter()
        .position(|value| value == "--no-pause")
        .is_some_and(|index| {
            arguments.remove(index);
            true
        });

    let result = if arguments.len() == 1 {
        execute(operation, Path::new(&arguments[0]))
    } else {
        Err(usage(operation).into())
    };

    let exit_code = match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[error] {error}");
            ExitCode::FAILURE
        }
    };
    if !no_pause {
        pause();
    }
    exit_code
}

fn execute(operation: DragOperation, input: &Path) -> Result<(), Box<dyn Error>> {
    match operation {
        DragOperation::XflUnpack => {
            let result = unpack_xfl_default(input)?;
            println!("[unpack] output={}", result.output.display());
            println!("[unpack] extracted_files={}", result.files);
            println!("[unpack] extracted_bytes={}", result.bytes);
        }
        DragOperation::XflPack => {
            let result = pack_xfl_default(input)?;
            println!("[pack] output={}", result.output.display());
            println!("[pack] packed_files={}", result.stats.packed_files);
            println!("[pack] packed_bytes={}", result.stats.packed_bytes);
            println!("[pack] used_manifest={}", result.stats.used_manifest);
        }
        DragOperation::GscExtract => {
            let result = extract_gsc_default(input)?;
            println!("[extract] output={}", result.output.display());
            println!("[extract] scanned_files={}", result.files);
            println!("[extract] extracted_entries={}", result.entries);
            println!(
                "[extract] skipped_text_records={}",
                result.skipped_text_records
            );
            println!("[extract] warnings=0");
        }
        DragOperation::GscInject => {
            let result = inject_gsc_default(input)?;
            println!("[inject] output={}", result.output.display());
            println!("[inject] rebuilt_files={}", result.files);
            println!("[inject] json_entries={}", result.entries);
            println!("[inject] changed_entries={}", result.changed_entries);
            println!("[inject] failed=0");
            println!("[inject] warnings=0");
        }
    }
    Ok(())
}

fn usage(operation: DragOperation) -> &'static str {
    match operation {
        DragOperation::XflUnpack => {
            "Drag one .xfl file onto xfl_unpack.exe\nCLI: xfl_unpack <archive.xfl> [--no-pause]"
        }
        DragOperation::XflPack => {
            "Drag one unpacked folder onto xfl_pack.exe\nCLI: xfl_pack <folder> [--no-pause]"
        }
        DragOperation::GscExtract => {
            "Drag one GSC folder onto gsc_extract.exe\nCLI: gsc_extract <gsc-folder> [--no-pause]"
        }
        DragOperation::GscInject => {
            "Drag one translated JSON folder onto gsc_inject.exe\nCLI: gsc_inject <json-folder> [--no-pause]"
        }
    }
}

fn pause() {
    println!();
    println!("Press Enter to close...");
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
}
