use anyhow::{Context, Result};
use random_house_d88_tool::basic::BasicProgram;
use random_house_d88_tool::d88::D88Image;
use std::{env, fs};

fn main() -> Result<()> {
    let source = env::args().nth(1).context("missing source D88")?;
    let image = D88Image::parse(fs::read(source)?)?;
    let mut old_clusters = 0usize;
    let mut compact_clusters = 0usize;
    for file in &image.files {
        if file.attribute != 0x80 {
            continue;
        }
        let compact = BasicProgram::parse(&file.data)?.compact_sector_rebuild()?;
        let clusters = compact.len().div_ceil(2048);
        old_clusters += file.clusters.len();
        compact_clusters += clusters;
        println!(
            "BASIC_COMPACT file={:?} old_bytes={} compact_bytes={} old_clusters={} compact_clusters={} released={}",
            file.name,
            file.data.len(),
            compact.len(),
            file.clusters.len(),
            clusters,
            file.clusters.len().saturating_sub(clusters)
        );
    }
    println!(
        "BASIC_COMPACT_TOTAL old_clusters={} compact_clusters={} released={}",
        old_clusters,
        compact_clusters,
        old_clusters - compact_clusters
    );
    Ok(())
}
