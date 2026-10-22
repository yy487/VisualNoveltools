use anyhow::{Context, Result};
use random_house_d88_tool::basic::BasicProgram;
use std::{env, fs};

fn main() -> Result<()> {
    let path = env::args()
        .nth(1)
        .context("usage: inspect_ram_control <lowram.bin>")?;
    let ram = fs::read(path)?;
    let program = BasicProgram::parse(&ram[1..])?;
    let mut address = 1u16;
    let mut addresses = Vec::new();
    for line in &program.lines {
        addresses.push((address, line.number));
        address = address.wrapping_add(u16::try_from(4 + line.body.len())?);
    }
    for (index, line) in program.lines.iter().enumerate() {
        if !matches!(line.number, 100 | 110 | 120 | 130 | 5000 | 5010) {
            continue;
        }
        let start = addresses[index].0;
        let refs = stored_references(&line.body)
            .into_iter()
            .map(|target| {
                let exact = addresses.iter().find(|(address, _)| *address == target);
                exact.map_or_else(
                    || format!("{target:04X}->NOT_LINE_START"),
                    |(_, number)| format!("{target:04X}->{number}"),
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "line={} address={start:04X} bytes={} refs={refs} raw={}",
            line.number,
            line.body.len(),
            line.body
                .iter()
                .map(|byte| format!("{byte:02X}"))
                .collect::<Vec<_>>()
                .join("_")
        );
    }
    Ok(())
}

fn stored_references(body: &[u8]) -> Vec<u16> {
    let mut refs = Vec::new();
    let mut quoted = false;
    let mut offset = 0usize;
    while offset + 2 < body.len() {
        let byte = body[offset];
        if byte == b'"' {
            quoted = !quoted;
            offset += 1;
        } else if !quoted && byte == 0x0D {
            refs.push(u16::from_le_bytes([body[offset + 1], body[offset + 2]]));
            offset += 3;
        } else {
            offset += 1;
        }
    }
    refs
}
