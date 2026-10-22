use anyhow::{Context, Result};
use random_house_d88_tool::d88::D88Image;
use std::collections::HashMap;
use std::{env, fs};

fn main() -> Result<()> {
    let disk_path = env::args()
        .nth(1)
        .context("usage: match_state_ram <disk.d88> <visible-ram.bin>")?;
    let ram_path = env::args()
        .nth(2)
        .context("usage: match_state_ram <disk.d88> <visible-ram.bin>")?;
    let image = D88Image::parse(fs::read(&disk_path)?)?;
    let ram = fs::read(&ram_path)?;
    if !matches!(ram.len(), 0x8000 | 0x10000) {
        anyhow::bail!("visible RAM dump must be exactly 32768 or 65536 bytes");
    }

    let ram_windows = ram.windows(16).enumerate().fold(
        HashMap::<[u8; 16], Vec<usize>>::new(),
        |mut map, (offset, window)| {
            map.entry(window.try_into().expect("16-byte window"))
                .or_default()
                .push(offset);
            map
        },
    );
    for file in &image.files {
        let prefix = file
            .data
            .iter()
            .take(24)
            .map(|byte| format!("{byte:02X}"))
            .collect::<Vec<_>>()
            .join("_");
        let (offset, run) = longest_ram_match(&file.data, &ram, &ram_windows);
        println!(
            "FILE name={:?} attr={:02X} bytes={} prefix={} ram_offset={} ram_run={}",
            file.name,
            file.attribute,
            file.data.len(),
            prefix,
            offset.map_or_else(|| "none".to_owned(), |value| format!("{value:04X}")),
            run,
        );
    }
    Ok(())
}

fn longest_ram_match(
    file: &[u8],
    ram: &[u8],
    ram_windows: &HashMap<[u8; 16], Vec<usize>>,
) -> (Option<usize>, usize) {
    let mut best_offset = None;
    let mut best_run = 0usize;
    for file_start in 0..file.len().saturating_sub(15) {
        if file.len() - file_start <= best_run {
            break;
        }
        let key: [u8; 16] = file[file_start..file_start + 16]
            .try_into()
            .expect("16-byte file window");
        for &ram_start in ram_windows.get(&key).into_iter().flatten() {
            let limit = (file.len() - file_start).min(ram.len() - ram_start);
            let mut run = 16usize;
            while run < limit && file[file_start + run] == ram[ram_start + run] {
                run += 1;
            }
            if run > best_run {
                best_run = run;
                best_offset = Some(ram_start);
            }
        }
    }
    (best_offset, best_run)
}
