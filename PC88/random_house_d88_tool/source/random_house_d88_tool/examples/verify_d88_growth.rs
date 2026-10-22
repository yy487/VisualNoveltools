use anyhow::{Context, Result};
use random_house_d88_tool::d88::D88Image;
use std::collections::HashMap;
use std::{env, fs};

fn main() -> Result<()> {
    let source = env::args().nth(1).context("missing source D88")?;
    let output = env::args().nth(2).context("missing output D88")?;
    let image = D88Image::parse(fs::read(&source)?)?;
    let original = image.file("MACHI")?;
    let donor = image.file("ME 1")?;
    let old_clusters = original.clusters.clone();
    let mut grown = original.data.clone();
    grown.resize(grown.len() + 2 * 1024, 0xA5);
    let shrunk = donor.data[..2048].to_vec();
    let mut replacements = HashMap::new();
    replacements.insert(original.name.clone(), grown.clone());
    replacements.insert(donor.name.clone(), shrunk.clone());
    let rebuilt = image.rebuild(&replacements)?;
    fs::write(&output, rebuilt.bytes)?;
    let verified = D88Image::parse(fs::read(&output)?)?;
    let actual = verified.file("MACHI")?;
    let actual_donor = verified.file("ME 1")?;
    anyhow::ensure!(actual.data == grown, "grown file data mismatch");
    anyhow::ensure!(actual_donor.data == shrunk, "donor file data mismatch");
    anyhow::ensure!(
        actual.clusters.starts_with(&old_clusters),
        "existing cluster prefix was not preserved"
    );
    println!(
        "D88_GROWTH_VERIFY pass=yes old_bytes={} new_bytes={} old_clusters={} new_clusters={} appended={:?} donor_old_clusters={} donor_new_clusters={}",
        original.data.len(),
        actual.data.len(),
        old_clusters.len(),
        actual.clusters.len(),
        &actual.clusters[old_clusters.len()..],
        donor.clusters.len(),
        actual_donor.clusters.len()
    );
    Ok(())
}
