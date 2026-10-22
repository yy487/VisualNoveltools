use anyhow::{Context, Result};
use encoding_rs::SHIFT_JIS;
use random_house_d88_tool::basic::BasicProgram;
use random_house_d88_tool::d88::D88Image;
use std::collections::BTreeMap;
use std::{env, fs};

const PRINT: u8 = 0x91;
const Q_ASSIGN: [u8; 3] = [b'Q', b'$', 0xF1];
const SC_ASSIGN: [u8; 4] = [b'S', b'C', b'$', 0xF1];
const MESSAGE_ASSIGN: [u8; 5] = [b'M', b'E', b'S', b'$', 0xF1];
const DATA: u8 = 0x84;

fn main() -> Result<()> {
    let path = env::args()
        .nth(1)
        .context("usage: inspect_text_candidates <disk.d88>")?;
    let file_filter = env::args().nth(2);
    let image = D88Image::parse(fs::read(path)?)?;
    let mut counts = BTreeMap::<&'static str, usize>::new();
    let mut samples = BTreeMap::<&'static str, usize>::new();
    let mut selected_unknown = BTreeMap::<u8, usize>::new();
    for file in &image.files {
        if file.attribute != 0x80 {
            continue;
        }
        if file_filter
            .as_ref()
            .is_some_and(|filter| !file.name.eq_ignore_ascii_case(filter))
        {
            continue;
        }
        let program = BasicProgram::parse(&file.data)?;
        for quoted in program.quoted_strings()? {
            let line = &program.lines[quoted.line_index];
            let before = &line.body[quoted.statement_offset..quoted.body_offset.saturating_sub(1)];
            let role = if before.contains(&PRINT) {
                "direct_print"
            } else if before
                .windows(Q_ASSIGN.len())
                .any(|window| window == Q_ASSIGN)
            {
                "message_q"
            } else if before
                .windows(SC_ASSIGN.len())
                .any(|window| window == SC_ASSIGN)
            {
                "message_sc"
            } else if before
                .windows(MESSAGE_ASSIGN.len())
                .any(|window| window == MESSAGE_ASSIGN)
            {
                "menu_prompt"
            } else if before.contains(&DATA) {
                "data_text"
            } else {
                "other"
            };
            *counts.entry(role).or_insert(0) += 1;
            if role != "other" {
                let mut had_unknown = false;
                for byte in &quoted.bytes {
                    if !is_supported_text_byte(*byte) {
                        had_unknown = true;
                        *selected_unknown.entry(*byte).or_insert(0) += 1;
                    }
                }
                if had_unknown {
                    eprintln!(
                        "UNKNOWN_SELECTED role={role} file={} line={} hex={}",
                        file.name,
                        quoted.line_number,
                        quoted
                            .bytes
                            .iter()
                            .map(|byte| format!("{byte:02X}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                }
            }
            let shown = samples.entry(role).or_insert(0);
            if *shown < 25 && (role != "other" || quoted.bytes.len() >= 16) {
                *shown += 1;
                if role == "other" {
                    let prefix = before
                        .iter()
                        .map(|byte| format!("{byte:02X}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    println!(
                        "{}\t{}\t{}\t0x{:X}\t{}\tPREFIX={}",
                        role,
                        file.name,
                        quoted.line_number,
                        quoted.file_offset,
                        decode_pc88(&quoted.bytes),
                        prefix
                    );
                } else {
                    println!(
                        "{}\t{}\t{}\t0x{:X}\t{}",
                        role,
                        file.name,
                        quoted.line_number,
                        quoted.file_offset,
                        decode_pc88(&quoted.bytes)
                    );
                }
            }
        }
    }
    eprintln!("CANDIDATE_COUNTS {counts:?}");
    eprintln!("SELECTED_UNKNOWN_BYTES {selected_unknown:02X?}");
    Ok(())
}

fn is_supported_text_byte(byte: u8) -> bool {
    matches!(
        byte,
        0x20..=0x7E
            | 0xA1..=0xDF
            | 0x80
            | 0x82
            | 0x83
            | 0x85
            | 0x87
            | 0x89
            | 0x8A
            | 0x8D
            | 0x8E
            | 0x8F
            | 0x9F
            | 0xE5
            | 0xE9
            | 0xEA
            | 0xF0
    )
}

fn decode_pc88(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| match *byte {
            0x20..=0x7E => char::from(*byte).to_string(),
            0xA1..=0xDF => {
                let raw = [*byte];
                let (text, _) = SHIFT_JIS.decode_without_bom_handling(&raw);
                text.into_owned()
            }
            0x80 => "▁".to_owned(),
            0x82 => "▃".to_owned(),
            0x83 => "▄".to_owned(),
            0x85 => "▆".to_owned(),
            0x87 => "█".to_owned(),
            0x89 => "▎".to_owned(),
            0x8A => "▍".to_owned(),
            0x8D => "▊".to_owned(),
            0x8E => "▉".to_owned(),
            0x8F => "┼".to_owned(),
            0x9F => "✓".to_owned(),
            0xE5 => "◣".to_owned(),
            0xE9 => "♥".to_owned(),
            0xEA => "◆".to_owned(),
            0xF0 => "╳".to_owned(),
            value => format!("⟦{value:02X}⟧"),
        })
        .collect()
}
