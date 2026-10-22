use anyhow::{bail, ensure, Context, Result};
use random_house_d88_tool::text::TextTranslation;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;

type StableKey = (String, u16, usize, usize, String);

fn main() -> Result<()> {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let old_path = args
        .next()
        .context("usage: rebase_translation <old.json> <fresh-extract.json> <output.json>")?;
    let fresh_path = args.next().context("missing fresh extraction path")?;
    let output_path = args.next().context("missing output path")?;
    ensure!(args.next().is_none(), "unexpected extra argument");
    if output_path.exists() {
        bail!("output already exists: {}", output_path.display());
    }

    let old: TextTranslation = serde_json::from_slice(&fs::read(&old_path)?)?;
    let mut fresh: TextTranslation = serde_json::from_slice(&fs::read(&fresh_path)?)?;
    let mut old_by_key = HashMap::new();
    for entry in old.entries {
        let key = stable_key(&entry);
        ensure!(
            old_by_key.insert(key, entry).is_none(),
            "old translation contains duplicate stable metadata"
        );
    }

    let mut seen = HashSet::new();
    let mut matched = 0usize;
    let mut translated = 0usize;
    for entry in &mut fresh.entries {
        let key = stable_key(entry);
        ensure!(
            seen.insert(key.clone()),
            "fresh extraction has a duplicate key"
        );
        if let Some(old_entry) = old_by_key.remove(&key) {
            entry.message = old_entry.message;
            matched += 1;
            translated += usize::from(entry.message != entry.scr_msg);
        }
    }
    ensure!(
        old_by_key
            .values()
            .all(|entry| entry.message == entry.scr_msg),
        "a translated old entry could not be matched to the fresh extraction"
    );

    let json = serde_json::to_vec_pretty(&fresh)?;
    fs::write(&output_path, json)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    println!(
        "matched={matched} translated={translated} added={} obsolete_untranslated={} output={}",
        fresh.entries.len() - matched,
        old_by_key.len(),
        output_path.display()
    );
    Ok(())
}

fn stable_key(entry: &random_house_d88_tool::text::TextEntry) -> StableKey {
    (
        entry.file.clone(),
        entry.line,
        entry.line_index,
        entry.body_offset,
        entry.scr_msg.clone(),
    )
}
