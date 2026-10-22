use anyhow::{Context, Result};
use random_house_d88_tool::basic::BasicProgram;
use random_house_d88_tool::d88::D88Image;
use std::{env, fs};

const TOKEN_REM: u8 = 0x8F;

fn main() -> Result<()> {
    let source = env::args().nth(1).context("missing source D88")?;
    let image = D88Image::parse(fs::read(source)?)?;
    for file in &image.files {
        if file.attribute != 0x80 {
            continue;
        }
        let program = BasicProgram::parse(&file.data)?;
        let mut removable = 0usize;
        let mut lines = 0usize;
        for line in &program.lines {
            let mut quoted = false;
            for (offset, byte) in line.body.iter().copied().enumerate() {
                if byte == 0 {
                    break;
                }
                if byte == b'"' {
                    quoted = !quoted;
                } else if !quoted && byte == TOKEN_REM {
                    let comment_end = line
                        .body
                        .iter()
                        .position(|value| *value == 0)
                        .unwrap_or(line.body.len());
                    removable += comment_end.saturating_sub(offset + 1);
                    lines += 1;
                    break;
                }
            }
        }
        if removable != 0 {
            println!(
                "REM_CAPACITY file={} lines={} removable_bytes={}",
                file.name, lines, removable
            );
        }
    }
    Ok(())
}
