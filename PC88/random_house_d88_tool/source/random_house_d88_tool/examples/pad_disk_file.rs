use anyhow::{bail, Context, Result};
use random_house_d88_tool::d88::D88Image;
use std::collections::HashMap;
use std::{env, fs, path::PathBuf};

fn main() -> Result<()> {
    let source = PathBuf::from(
        env::args()
            .nth(1)
            .context("usage: pad_disk_file <source.d88> <file> <output.d88>")?,
    );
    let file_name = env::args()
        .nth(2)
        .context("usage: pad_disk_file <source.d88> <file> <output.d88>")?;
    let output = PathBuf::from(
        env::args()
            .nth(3)
            .context("usage: pad_disk_file <source.d88> <file> <output.d88>")?,
    );
    if output.exists() {
        bail!("output already exists: {}", output.display());
    }

    let image = D88Image::parse(fs::read(&source)?)?;
    let file = image.file(&file_name)?;
    let capacity = file.clusters.len() * 8 * 256;
    if file.data.len() + 256 > capacity {
        bail!(
            "{} has no room for a zero padding sector: bytes={}, capacity={capacity}",
            file.name,
            file.data.len()
        );
    }
    let mut padded = file.data.clone();
    padded.resize(file.data.len() + 256, 0);
    let replacements = HashMap::from([(file.name.clone(), padded)]);
    let rebuilt = image.rebuild(&replacements)?;
    fs::write(&output, &rebuilt.bytes)
        .with_context(|| format!("write output {}", output.display()))?;
    println!(
        "PADDED file={:?} old_bytes={} new_bytes={} output={}",
        file.name,
        file.data.len(),
        file.data.len() + 256,
        output.display()
    );
    Ok(())
}
