use kokorov_fl2::{pack_archive, ToolError, ToolResult};
use std::env;
use std::path::PathBuf;

#[derive(Debug)]
struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
    overwrite: bool,
}

fn usage(program: &str) {
    println!(
        "KOKOROV FL2.0 packer\n\nUsage:\n  {program} [--output ARCHIVE.FL2] [--overwrite] UNPACKED_DIR\n\nUNPACKED_DIR must contain fl2_manifest.json produced by fl2_unpack.exe. The\npacker rebuilds payload order, the entry table, index_size, and index_offset.\nThe default output is a sibling <stem>_packed.FL2 file. The source directory is\nnever modified; existing output is refused unless --overwrite is supplied."
    );
}

fn parse_args() -> ToolResult<Args> {
    let mut raw = env::args_os().collect::<Vec<_>>();
    let program = raw
        .first()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "fl2_pack".to_string());
    raw.remove(0);
    let mut input = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 0usize;
    while index < raw.len() {
        let text = raw[index].to_string_lossy();
        match text.as_ref() {
            "-h" | "--help" => {
                usage(&program);
                std::process::exit(0);
            }
            "--overwrite" => overwrite = true,
            "-o" | "--output" => {
                index += 1;
                let value = raw
                    .get(index)
                    .ok_or_else(|| ToolError(format!("{text} requires an archive path")))?;
                output = Some(PathBuf::from(value));
            }
            _ if text.starts_with('-') => {
                return Err(ToolError(format!("unknown option '{text}'; use --help")))
            }
            _ => {
                if input.is_some() {
                    return Err(ToolError(
                        "only one unpacked directory is supported".to_string(),
                    ));
                }
                input = Some(PathBuf::from(&raw[index]));
            }
        }
        index += 1;
    }
    Ok(Args {
        input: input.ok_or_else(|| {
            usage(&program);
            ToolError("missing UNPACKED_DIR input".to_string())
        })?,
        output,
        overwrite,
    })
}

fn run() -> ToolResult<()> {
    let args = parse_args()?;
    let report = pack_archive(&args.input, args.output.as_deref(), args.overwrite)?;
    println!(
        "[pack] input={} packed_files={} output_bytes={} output={}",
        report.input.display(),
        report.packed_files,
        report.output_bytes,
        report.output.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("fl2_pack: error: {error}");
        std::process::exit(1);
    }
}
