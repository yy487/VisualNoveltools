use anyhow::{bail, Context, Result};
use random_house_d88_tool::renderer::resident_renderer;
use std::{env, fs, path::PathBuf};

fn main() -> Result<()> {
    let output = PathBuf::from(
        env::args()
            .nth(1)
            .context("usage: dump_resident_renderer <output.bin>")?,
    );
    if output.exists() {
        bail!("output already exists: {}", output.display());
    }
    fs::write(&output, resident_renderer()?)
        .with_context(|| format!("failed to write {}", output.display()))?;
    println!("renderer={}", output.display());
    Ok(())
}
