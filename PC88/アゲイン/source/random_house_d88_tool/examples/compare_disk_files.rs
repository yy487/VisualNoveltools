use anyhow::{Context, Result};
use random_house_d88_tool::basic::BasicProgram;
use random_house_d88_tool::d88::D88Image;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, fs};

fn main() -> Result<()> {
    let left_path = env::args()
        .nth(1)
        .context("usage: compare_disk_files <left.d88> <right.d88>")?;
    let right_path = env::args()
        .nth(2)
        .context("usage: compare_disk_files <left.d88> <right.d88>")?;
    let file_filter = env::args().nth(3);
    let left = D88Image::parse(fs::read(&left_path)?)?;
    let right = D88Image::parse(fs::read(&right_path)?)?;

    let right_files = right
        .files
        .iter()
        .map(|file| (file.name.to_ascii_uppercase(), file))
        .collect::<BTreeMap<_, _>>();

    println!("left={left_path}");
    println!("right={right_path}");
    for left_file in &left.files {
        if file_filter
            .as_ref()
            .is_some_and(|filter| !left_file.name.eq_ignore_ascii_case(filter))
        {
            continue;
        }
        let right_file = right_files
            .get(&left_file.name.to_ascii_uppercase())
            .with_context(|| format!("right disk has no file {}", left_file.name))?;
        let changed = left_file.data != right_file.data;
        let left_hash = hex_hash(&left_file.data);
        let right_hash = hex_hash(&right_file.data);
        println!(
            "FILE name={:?} attr={:02X} changed={} left_bytes={} right_bytes={} left_chain={} right_chain={} left_sha={} right_sha={}",
            left_file.name,
            left_file.attribute,
            changed,
            left_file.data.len(),
            right_file.data.len(),
            chain_text(&left_file.clusters),
            chain_text(&right_file.clusters),
            left_hash,
            right_hash
        );

        if changed && left_file.attribute == 0x80 && right_file.attribute == 0x80 {
            let left_program = BasicProgram::parse(&left_file.data)
                .with_context(|| format!("parse left BASIC {}", left_file.name))?;
            let right_program = BasicProgram::parse(&right_file.data)
                .with_context(|| format!("parse right BASIC {}", right_file.name))?;
            let left_compact = left_program.compact_unpadded_rebuild()?;
            let right_compact = right_program.compact_unpadded_rebuild()?;
            println!(
                "  BASIC_SIZE left_compact={} right_compact={} left_tail={} right_tail={}",
                left_compact.len(),
                right_compact.len(),
                tail_text(&left_compact, 32),
                tail_text(&right_compact, 32)
            );
            let left_lines = left_program
                .lines
                .iter()
                .map(|line| (line.number, &line.body))
                .collect::<BTreeMap<_, _>>();
            let right_lines = right_program
                .lines
                .iter()
                .map(|line| (line.number, &line.body))
                .collect::<BTreeMap<_, _>>();
            let numbers = left_lines
                .keys()
                .chain(right_lines.keys())
                .copied()
                .collect::<BTreeSet<_>>();
            let differences = numbers
                .into_iter()
                .filter(|number| left_lines.get(number) != right_lines.get(number))
                .map(|number| {
                    format!(
                        "{}:{}>{}",
                        number,
                        left_lines.get(&number).map_or(0, |body| body.len()),
                        right_lines.get(&number).map_or(0, |body| body.len())
                    )
                })
                .collect::<Vec<_>>();
            println!("  BASIC_DIFF {}", differences.join(","));
        }
    }
    Ok(())
}

fn hex_hash(bytes: &[u8]) -> String {
    format!("{:X}", Sha256::digest(bytes))
}

fn chain_text(chain: &[u8]) -> String {
    chain
        .iter()
        .map(|cluster| format!("{cluster:02X}"))
        .collect::<Vec<_>>()
        .join("-")
}

fn tail_text(bytes: &[u8], count: usize) -> String {
    bytes[bytes.len().saturating_sub(count)..]
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join("_")
}
