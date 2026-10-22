use anyhow::{Context, Result};
use random_house_d88_tool::basic::BasicProgram;
use random_house_d88_tool::d88::D88Image;
use std::{env, fs};

const TOKEN_REM: u8 = 0x8F;
const TOKEN_CLS: u8 = 0xCE;
const TOKEN_POLL: [u8; 2] = [0xFF, 0xDF];

fn main() -> Result<()> {
    let path = env::args()
        .nth(1)
        .context("usage: inspect_cls_contexts <disk.d88>")?;
    let image = D88Image::parse(fs::read(path)?)?;
    let mut total = 0usize;
    let mut poll_arguments = 0usize;
    for file in &image.files {
        if file.attribute != 0x80 {
            continue;
        }
        let program = BasicProgram::parse(&file.data)?;
        for line in &program.lines {
            let content = &line.body[..line.body.len() - 1];
            let mut quoted = false;
            let mut statement_start = 0usize;
            let mut previous_statement = Vec::new();
            let mut offset = 0usize;
            while offset < content.len() {
                let byte = content[offset];
                if byte == b'"' {
                    quoted = !quoted;
                    offset += 1;
                } else if quoted {
                    offset += 1;
                } else if byte == TOKEN_REM {
                    break;
                } else if byte == b':' {
                    previous_statement = content[statement_start..offset].to_vec();
                    statement_start = offset + 1;
                    offset += 1;
                } else if byte == TOKEN_CLS {
                    total += 1;
                    let statement = &content[statement_start..offset];
                    let poll = statement
                        .windows(TOKEN_POLL.len())
                        .any(|window| window == TOKEN_POLL);
                    poll_arguments += usize::from(poll);
                    println!(
                        "file={} line={} offset={} poll={} previous={} statement={}",
                        file.name,
                        line.number,
                        offset,
                        poll,
                        previous_statement
                            .iter()
                            .map(|byte| format!("{byte:02X}"))
                            .collect::<Vec<_>>()
                            .join("_"),
                        statement
                            .iter()
                            .map(|byte| format!("{byte:02X}"))
                            .collect::<Vec<_>>()
                            .join("_")
                    );
                    offset += 1;
                } else {
                    offset += 1 + token_payload_len(byte);
                }
            }
        }
    }
    println!(
        "CLS_CONTEXT total={total} poll_arguments={poll_arguments} executable={}",
        total - poll_arguments
    );
    Ok(())
}

fn token_payload_len(token: u8) -> usize {
    match token {
        0x0B..=0x0E => 2,
        0x0F => 1,
        0x1C => 2,
        0x1D => 4,
        0x1F => 8,
        0xFF => 1,
        _ => 0,
    }
}
