use ai5win_tools::{arc, g24, msk, pipeline};
use anyhow::{bail, ensure, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use image::GenericImageView;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    version,
    about = "AI5WIN ARC/G24/MSK tools; files and directories may be dragged onto the executable"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    ArcUnpack {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    ArcPack {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    ArcVerify {
        source: PathBuf,
        rebuilt: PathBuf,
    },
    G24Decode {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    G24Encode {
        input: PathBuf,
        output: Option<PathBuf>,
        #[arg(long)]
        reference: Option<PathBuf>,
        #[arg(long, default_value_t = 0)]
        x: i16,
        #[arg(long, default_value_t = 0)]
        y: i16,
    },
    MskDecode {
        input: PathBuf,
        output: Option<PathBuf>,
        #[arg(long, requires = "height")]
        width: Option<u32>,
        #[arg(long, requires = "width")]
        height: Option<u32>,
    },
    MskEncode {
        input: PathBuf,
        output: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = MaskMode::Raw)]
        kind: MaskMode,
    },
    BuildPatch {
        #[arg(long)]
        bg: PathBuf,
        #[arg(long)]
        data: PathBuf,
        #[arg(long)]
        edits: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum MaskMode {
    Raw,
    TypeA,
    TitlePt,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        return run_command(command);
    }
    ensure!(
        !cli.paths.is_empty(),
        "provide a command or drag one or more paths onto the executable"
    );
    for path in cli.paths {
        auto_process(&path)
            .with_context(|| format!("automatic processing failed: {}", path.display()))?;
    }
    Ok(())
}

fn run_command(command: Command) -> Result<()> {
    match command {
        Command::ArcUnpack { input, output } => {
            let output = output.unwrap_or_else(|| sibling_with_suffix(&input, "_unpacked"));
            let entries = arc::unpack(&input, &output)?;
            println!("unpacked {} entries to {}", entries.len(), output.display());
        }
        Command::ArcPack { input, output } => {
            let output = output.unwrap_or_else(|| input.with_extension("arc"));
            let entries = arc::pack(&input, &output)?;
            println!("packed {} entries to {}", entries.len(), output.display());
        }
        Command::ArcVerify { source, rebuilt } => {
            let result = arc::verify_repack(&source, &rebuilt, &HashMap::new())?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Command::G24Decode { input, output } => {
            let output = output.unwrap_or_else(|| input.with_extension("png"));
            let header = g24::decode_file(&input, &output)?;
            println!(
                "decoded {}x{} to {}",
                header.width,
                header.height,
                output.display()
            );
        }
        Command::G24Encode {
            input,
            output,
            reference,
            mut x,
            mut y,
        } => {
            if let Some(reference) = reference {
                let data = fs::read(reference)?;
                let header = g24::read_header(&data)?;
                x = header.x;
                y = header.y;
            }
            let output = output.unwrap_or_else(|| input.with_extension("G24"));
            let header = g24::encode_file(&input, &output, x, y)?;
            println!(
                "encoded {}x{} to {}",
                header.width,
                header.height,
                output.display()
            );
        }
        Command::MskDecode {
            input,
            output,
            width,
            height,
        } => {
            let output = output.unwrap_or_else(|| input.with_extension("png"));
            let dimensions = width.zip(height);
            let kind = msk::decode_file(&input, &output, dimensions)?;
            println!("decoded {kind:?} to {}", output.display());
        }
        Command::MskEncode {
            input,
            output,
            kind,
        } => {
            let image = image::open(&input)?;
            let dimensions = image.dimensions();
            let msk_kind = match kind {
                MaskMode::Raw => msk::MskKind::Raw8 {
                    width: dimensions.0,
                    height: dimensions.1,
                },
                MaskMode::TypeA => {
                    ensure!(
                        dimensions.0 <= u16::MAX as u32 && dimensions.1 <= u16::MAX as u32,
                        "Type A dimensions exceed u16"
                    );
                    msk::MskKind::TypeA {
                        width: dimensions.0 as u16,
                        height: dimensions.1 as u16,
                    }
                }
                MaskMode::TitlePt => msk::MskKind::TitlePt,
            };
            let output = output.unwrap_or_else(|| {
                if matches!(kind, MaskMode::TitlePt) {
                    input.with_file_name("TITLE_PT_M.MSK")
                } else {
                    input.with_extension("MSK")
                }
            });
            msk::encode_file(&input, &output, msk_kind)?;
            println!("encoded {msk_kind:?} to {}", output.display());
        }
        Command::BuildPatch {
            bg,
            data,
            edits,
            output,
        } => {
            let report = pipeline::build_patch(&bg, &data, &edits, &output)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }
    Ok(())
}

fn auto_process(path: &Path) -> Result<()> {
    if path.is_dir() {
        let parent = path.parent().unwrap_or(path);
        if let (Some(bg), Some(data)) = (
            find_case_insensitive(parent, "Bg.arc"),
            find_case_insensitive(parent, "data.arc"),
        ) {
            let report = pipeline::build_patch(&bg, &data, path, parent)?;
            println!("built {} and {}", report.output_bg, report.output_data);
            return Ok(());
        }
        let output = path.with_extension("arc");
        arc::pack(path, &output)?;
        println!("packed directory to {}", output.display());
        return Ok(());
    }

    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "arc" => {
            let output = sibling_with_suffix(path, "_unpacked");
            arc::unpack(path, &output)?;
            println!("unpacked ARC to {}", output.display());
        }
        "g24" => {
            let output = path.with_extension("png");
            g24::decode_file(path, &output)?;
            println!("decoded G24 to {}", output.display());
        }
        "msk" => {
            let output = path.with_extension("png");
            msk::decode_file(path, &output, None)?;
            println!("decoded MSK to {}", output.display());
        }
        "png" => {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if stem.to_ascii_uppercase().ends_with("_M") || msk::is_title_name(name) {
                let image = image::open(path)?;
                let dimensions = image.dimensions();
                let (output, kind) = if msk::is_title_name(name) {
                    (path.with_file_name("TITLE_PT_M.MSK"), msk::MskKind::TitlePt)
                } else {
                    (
                        path.with_extension("MSK"),
                        msk::MskKind::Raw8 {
                            width: dimensions.0,
                            height: dimensions.1,
                        },
                    )
                };
                msk::encode_file(path, &output, kind)?;
                println!("encoded mask to {}", output.display());
            } else {
                let output = path.with_extension("G24");
                g24::encode_file(path, &output, 0, 0)?;
                println!("encoded image to {}", output.display());
            }
        }
        _ => bail!("unsupported drag-and-drop path: {}", path.display()),
    }
    Ok(())
}

fn sibling_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("output");
    path.with_file_name(format!("{stem}{suffix}"))
}

fn find_case_insensitive(directory: &Path, wanted: &str) -> Option<PathBuf> {
    fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case(wanted))
        })
}
