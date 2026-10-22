use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::{env, fs, path::PathBuf};

fn main() -> Result<()> {
    let source = PathBuf::from(env::args().nth(1).context(
        "usage: make_titl_translation_probe <source.json> <max_titl_line> <output.json>",
    )?);
    let max_line = env::args()
        .nth(2)
        .context("usage: make_titl_translation_probe <source.json> <max_titl_line> <output.json>")?
        .parse::<u64>()?;
    let output = PathBuf::from(env::args().nth(3).context(
        "usage: make_titl_translation_probe <source.json> <max_titl_line> <output.json>",
    )?);
    if output.exists() {
        bail!("output already exists: {}", output.display());
    }

    let mut root: Value = serde_json::from_slice(&fs::read(&source)?)?;
    let entries = root
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .context("translation JSON has no entries array")?;
    let mut reset = 0usize;
    let mut kept = 0usize;
    for entry in entries {
        if entry.get("_file").and_then(Value::as_str) != Some("TITL") {
            continue;
        }
        let line = entry
            .get("_line")
            .and_then(Value::as_u64)
            .context("TITL entry has no numeric _line")?;
        if line <= max_line {
            kept += 1;
        } else {
            let source_text = entry
                .get("scr_msg")
                .and_then(Value::as_str)
                .context("TITL entry has no scr_msg")?
                .to_owned();
            entry["message"] = Value::String(source_text);
            reset += 1;
        }
    }
    let mut encoded = serde_json::to_vec_pretty(&root)?;
    encoded.push(b'\n');
    fs::write(&output, encoded)?;
    println!(
        "TITL_PROBE max_line={max_line} kept={kept} reset={reset} output={}",
        output.display()
    );
    Ok(())
}
