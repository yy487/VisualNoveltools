use kokorov_fl2::{parse_archive, unpack_archive, ToolError, ToolResult};
use std::env;
use std::path::PathBuf;

#[derive(Debug)]
struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
    list: bool,
    overwrite: bool,
}

fn usage(program: &str) {
    println!(
        "KOKOROV FL2.0 unpacker\n\nUsage:\n  {program} [--output DIR] [--list] [--overwrite] ARCHIVE.FL2\n\nOptions:\n  -o, --output DIR  output directory (default: ARCHIVE_stem_unpacked)\n      --list        validate and list entries without writing files\n      --overwrite    allow existing output files/directories\n  -h, --help        show this help\n\nThe input archive is never modified. The unpacker writes extracted payloads and\nfl2_manifest.json; use fl2_pack.exe to rebuild an archive."
    );
}

fn parse_args() -> ToolResult<Args> {
    let mut raw = env::args_os().collect::<Vec<_>>();
    let program = raw
        .first()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "fl2_unpack".to_string());
    raw.remove(0);
    let mut input = None;
    let mut output = None;
    let mut list = false;
    let mut overwrite = false;
    let mut index = 0usize;
    while index < raw.len() {
        let text = raw[index].to_string_lossy();
        match text.as_ref() {
            "-h" | "--help" => {
                usage(&program);
                std::process::exit(0);
            }
            "--list" => list = true,
            "--overwrite" => overwrite = true,
            "-o" | "--output" => {
                index += 1;
                let value = raw
                    .get(index)
                    .ok_or_else(|| ToolError(format!("{text} requires a directory")))?;
                output = Some(PathBuf::from(value));
            }
            _ if text.starts_with('-') => {
                return Err(ToolError(format!("unknown option '{text}'; use --help")))
            }
            _ => {
                if input.is_some() {
                    return Err(ToolError("only one input archive is supported".to_string()));
                }
                input = Some(PathBuf::from(&raw[index]));
            }
        }
        index += 1;
    }
    Ok(Args {
        input: input.ok_or_else(|| {
            usage(&program);
            ToolError("missing ARCHIVE.FL2 input".to_string())
        })?,
        output,
        list,
        overwrite,
    })
}

fn run() -> ToolResult<()> {
    let args = parse_args()?;
    let bytes = std::fs::read(&args.input)
        .map_err(|error| ToolError(format!("cannot read '{}': {error}", args.input.display())))?;
    let archive = parse_archive(&bytes)?;
    if args.list {
        println!(
            "[list] input={} bytes={} magic=FL2.0 header_size={} entries={} index_offset={} index_size={}",
            args.input.display(),
            archive.file_len,
            archive.header.header_size,
            archive.entries.len(),
            archive.header.index_offset,
            archive.header.index_size
        );
        for entry in &archive.entries {
            println!(
                "[list] index={} offset=0x{:x} size={} name={}",
                entry.index, entry.data_offset, entry.size, entry.name
            );
        }
        return Ok(());
    }

    let report = unpack_archive(&args.input, args.output.as_deref(), args.overwrite)?;
    println!(
        "[unpack] input={} extracted_files={} extracted_bytes={} output={} manifest={}",
        report.input.display(),
        report.extracted_files,
        report.extracted_bytes,
        report.output.display(),
        report.manifest.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("fl2_unpack: error: {error}");
        std::process::exit(1);
    }
}
