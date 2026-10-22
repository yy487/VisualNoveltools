use anyhow::{Context, Result};
use random_house_d88_tool::basic::BasicProgram;
use random_house_d88_tool::d88::D88Image;
use std::{env, fs};

fn main() -> Result<()> {
    let path = env::args()
        .nth(1)
        .context("usage: inspect_tokens <disk.d88> [file] [min_line] [max_line]")?;
    let file_filter = env::args().nth(2);
    let min_line = env::args().nth(3).map(|value| value.parse()).transpose()?;
    let max_line = env::args().nth(4).map(|value| value.parse()).transpose()?;
    let image = D88Image::parse(fs::read(path)?)?;
    for file in &image.files {
        if file.attribute != 0x80
            || file_filter
                .as_ref()
                .is_some_and(|filter| filter != "*" && !file.name.eq_ignore_ascii_case(filter))
        {
            continue;
        }
        let program = BasicProgram::parse(&file.data)?;
        for line in program.lines {
            let has_range = min_line.is_some() || max_line.is_some();
            let in_range = min_line.is_none_or(|minimum| line.number >= minimum)
                && max_line.is_none_or(|maximum| line.number <= maximum);
            if (has_range && in_range)
                || (!has_range
                    && line
                        .body
                        .iter()
                        .any(|byte| matches!(byte, 0x82 | 0x83 | 0x92 | 0x98 | 0xDC | 0xF0..=0xF8)))
            {
                let hex = line
                    .body
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("{}:{} {}", file.name, line.number, hex);
            }
        }
    }
    Ok(())
}
