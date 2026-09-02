use foxy2_pd7::pd7::{self, DecodedImage, SCREEN_HEIGHT, SCREEN_WIDTH};
use foxy2_pd7::png;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

type AppResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Copy)]
struct DecodeOptions {
    crop: bool,
    overwrite: bool,
}

fn usage() {
    println!(
        "foxy2-pd7 0.1.0\n\
         Decode elf FOXY2 PD7 graphics to PNG.\n\n\
         USAGE:\n\
           foxy2-pd7 decode <INPUT> <OUTPUT> [--crop] [--overwrite]\n\
           foxy2-pd7 inspect <INPUT>\n\
           foxy2-pd7 [INPUT]\n\n\
         INPUT may be one PD7 file or a directory. Directory decoding is recursive\n\
         and preserves relative paths. The default PNG canvas is 640x400; --crop\n\
         trims it to the union of the decoded blocks. A no-argument or path-only\n\
         invocation opens an interactive confirmation flow."
    );
}

fn is_pd7(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pd7"))
}

fn collect_pd7(path: &Path, output: &mut Vec<PathBuf>) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_file() {
        if is_pd7(path) {
            output.push(path.to_path_buf());
        }
        return Ok(());
    }
    if !metadata.is_dir() {
        return Ok(());
    }
    let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let file_type = entry.file_type()?;
        if file_type.is_dir() || (file_type.is_file() && is_pd7(&entry.path())) {
            collect_pd7(&entry.path(), output)?;
        }
    }
    Ok(())
}

fn load_decode(path: &Path) -> AppResult<DecodedImage> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    pd7::decode(&bytes).map_err(|error| format!("{}: {error}", path.display()).into())
}

fn output_for(input_root: &Path, output_root: &Path, input: &Path) -> AppResult<PathBuf> {
    if input_root.is_file() {
        return Ok(output_root.to_path_buf());
    }
    let relative = input.strip_prefix(input_root)?;
    let mut output = output_root.join(relative);
    output.set_extension("png");
    Ok(output)
}

fn same_existing_file(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn write_image(path: &Path, image: &DecodedImage, crop: bool, overwrite: bool) -> AppResult<()> {
    if path.exists() && !overwrite {
        return Err(format!("output exists (use --overwrite): {}", path.display()).into());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cropped = crop.then(|| image.cropped_rgba());
    let (width, height, pixels) = match &cropped {
        Some((width, height, pixels)) => (*width, *height, pixels.as_slice()),
        None => (
            u32::try_from(SCREEN_WIDTH).unwrap(),
            u32::try_from(SCREEN_HEIGHT).unwrap(),
            image.rgba.as_slice(),
        ),
    };

    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("output.png");
    let temporary = path.with_file_name(format!(".{file_name}.{}.tmp", std::process::id()));
    png::write_rgba(&temporary, width, height, pixels)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    Ok(())
}

fn decode_path(input: &Path, output: &Path, options: DecodeOptions) -> AppResult<()> {
    if !input.exists() {
        return Err(format!("input does not exist: {}", input.display()).into());
    }
    if input.is_file() && same_existing_file(input, output) {
        return Err("input and output resolve to the same file".into());
    }

    let mut inputs = Vec::new();
    collect_pd7(input, &mut inputs)?;
    if inputs.is_empty() {
        return Err(format!("no PD7 files found under {}", input.display()).into());
    }

    for source in &inputs {
        load_decode(source)?;
        let destination = output_for(input, output, source)?;
        if destination.exists() && !options.overwrite {
            return Err(
                format!("output exists (use --overwrite): {}", destination.display()).into(),
            );
        }
    }

    let mut blocks = 0_usize;
    let mut trailing = 0_usize;
    for source in &inputs {
        let image = load_decode(source)?;
        blocks += image.blocks.len();
        trailing += image
            .blocks
            .iter()
            .map(foxy2_pd7::pd7::BlockInfo::trailing_bytes)
            .sum::<usize>();
        let destination = output_for(input, output, source)?;
        write_image(&destination, &image, options.crop, options.overwrite)?;
    }

    println!(
        "Decoded {} PD7 file(s), {} block(s), {} trailing pad/EOF byte(s). Output: {}",
        inputs.len(),
        blocks,
        trailing,
        output.display()
    );
    Ok(())
}

fn inspect_path(input: &Path) -> AppResult<()> {
    let mut inputs = Vec::new();
    collect_pd7(input, &mut inputs)?;
    if inputs.is_empty() {
        return Err(format!("no PD7 files found under {}", input.display()).into());
    }

    let mut block_count = 0_usize;
    let mut trailing = 0_usize;
    for source in &inputs {
        let image = load_decode(source)?;
        println!(
            "{}: split={}, blocks={}",
            source.display(),
            image
                .split_offset
                .map_or_else(|| "none".to_owned(), |value| format!("0x{value:X}")),
            image.blocks.len()
        );
        for (index, block) in image.blocks.iter().enumerate() {
            println!(
                "  block {index}: offset=0x{:X}, x={}, y={}, width={} px, height={} (stored {}), stream=0x{:X}..0x{:X}, trailing={}",
                block.offset,
                block.x_pixels(),
                block.y,
                block.width_pixels(),
                block.decoded_height,
                block.stored_height,
                block.stream_offset,
                block.stream_end,
                block.trailing_bytes()
            );
            trailing += block.trailing_bytes();
        }
        block_count += image.blocks.len();
    }
    println!(
        "Inspected {} PD7 file(s), {} block(s), {} trailing pad/EOF byte(s).",
        inputs.len(),
        block_count,
        trailing
    );
    Ok(())
}

fn prompt(label: &str, default: Option<&str>) -> io::Result<String> {
    match default {
        Some(value) => print!("{label} [{value}]: "),
        None => print!("{label}: "),
    }
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let trimmed = line.trim();
    Ok(if trimmed.is_empty() {
        default.unwrap_or_default().to_owned()
    } else {
        trimmed.to_owned()
    })
}

fn yes_no(label: &str, default: bool) -> io::Result<bool> {
    let suffix = if default { "Y/n" } else { "y/N" };
    let answer = prompt(&format!("{label} ({suffix})"), None)?;
    if answer.is_empty() {
        return Ok(default);
    }
    Ok(matches!(answer.to_ascii_lowercase().as_str(), "y" | "yes"))
}

fn suggested_output(input: &Path) -> PathBuf {
    if input.is_file() {
        return input.with_extension("png");
    }
    let name = input
        .file_name()
        .and_then(OsStr::to_str)
        .map_or_else(|| "pd7_png".to_owned(), |name| format!("{name}_png"));
    input.parent().unwrap_or_else(|| Path::new(".")).join(name)
}

fn interactive(prefill: Option<&Path>) -> AppResult<()> {
    println!("Interactive PD7 to PNG conversion (nothing is written before confirmation).\n");
    let default_input = prefill.as_ref().map(|path| path.to_string_lossy());
    let input_text = prompt("Input PD7 file or directory", default_input.as_deref())?;
    if input_text.is_empty() {
        return Err("input path is required".into());
    }
    let input = PathBuf::from(input_text);
    let suggestion = suggested_output(&input);
    let output_text = prompt(
        "Output PNG file or directory",
        Some(&suggestion.to_string_lossy()),
    )?;
    let output = PathBuf::from(output_text);
    let crop = yes_no("Crop to decoded block bounds", false)?;
    let overwrite = yes_no("Allow replacing existing output files", false)?;
    println!(
        "\nInput: {}\nOutput: {}\nCrop: {}\nOverwrite: {}",
        input.display(),
        output.display(),
        crop,
        overwrite
    );
    if !yes_no("Proceed", false)? {
        println!("Cancelled.");
        return Ok(());
    }
    decode_path(&input, &output, DecodeOptions { crop, overwrite })
}

fn run() -> AppResult<()> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.is_empty() {
        return interactive(None);
    }
    if arguments.len() == 1 && !matches!(arguments[0].as_str(), "-h" | "--help") {
        return interactive(Some(Path::new(&arguments[0])));
    }
    match arguments[0].as_str() {
        "-h" | "--help" => {
            usage();
            Ok(())
        }
        "inspect" => {
            if arguments.len() != 2 {
                return Err("inspect requires exactly one INPUT path".into());
            }
            inspect_path(Path::new(&arguments[1]))
        }
        "decode" => {
            if arguments.len() < 3 {
                return Err("decode requires INPUT and OUTPUT paths".into());
            }
            let mut options = DecodeOptions {
                crop: false,
                overwrite: false,
            };
            for argument in &arguments[3..] {
                match argument.as_str() {
                    "--crop" => options.crop = true,
                    "--overwrite" => options.overwrite = true,
                    _ => return Err(format!("unknown option: {argument}").into()),
                }
            }
            decode_path(Path::new(&arguments[1]), Path::new(&arguments[2]), options)
        }
        command => Err(format!("unknown command: {command}").into()),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
