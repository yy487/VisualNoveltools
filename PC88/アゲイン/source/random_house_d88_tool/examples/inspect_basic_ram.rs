use anyhow::{Context, Result};
use random_house_d88_tool::basic::BasicProgram;
use random_house_d88_tool::d88::D88Image;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const BASIC_ATTRIBUTE: u8 = 0x80;
const CLS_TOKEN: u8 = 0xCE;

fn main() -> Result<()> {
    let ram_path = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .context("usage: inspect_basic_ram <lowram.bin> <disk.d88>")?;
    let disk_path = env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .context("usage: inspect_basic_ram <lowram.bin> <disk.d88>")?;
    let ram = fs::read(&ram_path)
        .with_context(|| format!("failed to read RAM dump {}", ram_path.display()))?;
    let ram_program = BasicProgram::parse(
        ram.get(1..)
            .context("RAM dump does not contain the N88 BASIC base address")?,
    )
    .context("failed to parse RAM BASIC program")?;
    let disk = D88Image::parse(
        fs::read(&disk_path)
            .with_context(|| format!("failed to read D88 {}", disk_path.display()))?,
    )?;

    let mut matches = Vec::new();
    for file in &disk.files {
        if file.attribute != BASIC_ATTRIBUTE {
            continue;
        }
        let program = BasicProgram::parse(&file.data)
            .with_context(|| format!("failed to parse BASIC file {}", file.name))?;
        if program
            .lines
            .iter()
            .map(|line| line.number)
            .eq(ram_program.lines.iter().map(|line| line.number))
        {
            matches.push(file.name.clone());
        }
    }

    println!(
        "RAM_BASIC lines={} compact_bytes={} matches={}",
        ram_program.lines.len(),
        ram_program
            .lines
            .iter()
            .map(|line| 4 + line.body.len())
            .sum::<usize>()
            + 2,
        if matches.is_empty() {
            "<none>".to_owned()
        } else {
            matches.join(",")
        }
    );

    let mut line_addresses = BTreeMap::new();
    let mut address = 1usize;
    for line in &ram_program.lines {
        line_addresses.insert(address as u16, line.number);
        address += 4 + line.body.len();
    }

    for line in &ram_program.lines {
        let content_end = line
            .body
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(line.body.len());
        let content = &line.body[..content_end];
        let has_yn = content.windows(3).any(|window| window == b"Y/N");
        let has_cls = outside_literals(content).contains(&CLS_TOKEN);
        if has_yn || has_cls {
            let references = stored_line_references(content)
                .into_iter()
                .map(|address| {
                    let target = line_addresses
                        .range(..=address)
                        .next_back()
                        .map(|(start, line)| format!("{line}+{:X}", address - start))
                        .unwrap_or_else(|| "?".to_owned());
                    format!("{address:04X}->{target}")
                })
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "line={} yn={} cls={} refs={} raw={}",
                line.number,
                has_yn,
                has_cls,
                references,
                content
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
    Ok(())
}

fn stored_line_references(content: &[u8]) -> Vec<u16> {
    let mut output = Vec::new();
    let mut quoted = false;
    let mut index = 0usize;
    while index < content.len() {
        let byte = content[index];
        if byte == b'"' {
            quoted = !quoted;
            index += 1;
        } else if !quoted && byte == 0x0D && index + 2 < content.len() {
            output.push(u16::from_le_bytes([content[index + 1], content[index + 2]]));
            index += 3;
        } else {
            index += 1;
        }
    }
    output
}

fn outside_literals(content: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(content.len());
    let mut quoted = false;
    for byte in content.iter().copied() {
        if byte == b'"' {
            quoted = !quoted;
        } else if !quoted {
            output.push(byte);
        }
    }
    output
}
