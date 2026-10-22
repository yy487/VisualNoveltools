use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::{env, fs, path::PathBuf};

const SAFE_LEADS: [u8; 16] = [
    0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE6, 0xE7, 0xE8, 0xEB, 0xEC, 0xED, 0xEE, 0xEF, 0xF6, 0xF7, 0xF8,
];

fn main() -> Result<()> {
    let source = PathBuf::from(
        env::args()
            .nth(1)
            .context("usage: remap_font_map_printable_trails <source.json> <output.json>")?,
    );
    let output = PathBuf::from(
        env::args()
            .nth(2)
            .context("usage: remap_font_map_printable_trails <source.json> <output.json>")?,
    );
    if output.exists() {
        bail!("output already exists: {}", output.display());
    }
    let mut root: Value = serde_json::from_slice(&fs::read(&source)?)?;
    let entries = root
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .context("font map has no entries array")?;
    let candidates = SAFE_LEADS.into_iter().flat_map(|lead| {
        (0x20u8..=0x7E)
            .filter(|trail| *trail != b'"')
            .map(move |trail| (lead, trail))
    });
    for (entry, (lead, trail)) in entries.iter_mut().zip(candidates) {
        let lead_index = SAFE_LEADS
            .iter()
            .position(|candidate| *candidate == lead)
            .context("safe lead disappeared")?;
        let slot = (lead_index << 8) | usize::from(trail);
        entry["carrier"] = Value::String(format!("{lead:02X}{trail:02X}"));
        entry["_slot"] = Value::from(slot);
        entry["_rom_offset"] = Value::from(slot * 32);
    }
    let capacity = SAFE_LEADS.len() * (0x7F - 0x20 - 1);
    if entries.len() > capacity {
        bail!(
            "font map has {} entries, exceeding printable-trail capacity {capacity}",
            entries.len()
        );
    }
    let entry_count = entries.len();
    let mut encoded = serde_json::to_vec_pretty(&root)?;
    encoded.push(b'\n');
    fs::write(&output, encoded)?;
    println!(
        "PRINTABLE_TRAIL_REMAP entries={} capacity={capacity} output={}",
        entry_count,
        output.display()
    );
    Ok(())
}
