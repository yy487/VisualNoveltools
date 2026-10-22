use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq, Hash)]
struct Entry {
    #[serde(default)]
    name: String,
    #[serde(default)]
    message: String,
}

#[derive(Debug, Default)]
struct Stats {
    entries: usize,
    named_entries: usize,
    message_chars: usize,
    name_chars: usize,
    empty_messages: usize,
    unique_entries: usize,
    unique_messages: usize,
}

#[derive(Debug)]
struct DiffBlock {
    old_start: usize,
    new_start: usize,
    deletes: Vec<Entry>,
    inserts: Vec<Entry>,
}

fn usage(program: &str) -> String {
    format!(
        "usage: {program} [old.json] [new.json] [report.md]\n\
         default: {program} all.orig.json all.orig_new.json mes_diff_report.md"
    )
}

fn read_entries(path: &Path) -> Result<Vec<Entry>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str::<Vec<Entry>>(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn stats(entries: &[Entry]) -> Stats {
    let mut entry_counts = HashMap::<Entry, usize>::new();
    let mut msg_counts = HashMap::<&str, usize>::new();
    for e in entries {
        *entry_counts.entry(e.clone()).or_default() += 1;
        *msg_counts.entry(e.message.as_str()).or_default() += 1;
    }
    Stats {
        entries: entries.len(),
        named_entries: entries.iter().filter(|e| !e.name.is_empty()).count(),
        message_chars: entries.iter().map(|e| e.message.chars().count()).sum(),
        name_chars: entries.iter().map(|e| e.name.chars().count()).sum(),
        empty_messages: entries.iter().filter(|e| e.message.is_empty()).count(),
        unique_entries: entry_counts.len(),
        unique_messages: msg_counts.len(),
    }
}

fn entry_key(e: &Entry) -> String {
    format!("{}\u{1f}{}", e.name, e.message)
}

fn encode_line(e: &Entry) -> String {
    serde_json::to_string(e).expect("entry serialization failed")
}

fn decode_line(line: &str) -> Result<Entry, String> {
    serde_json::from_str(line).map_err(|e| format!("internal diff line decode failed: {e}: {line}"))
}

fn preview(text: &str, max_chars: usize) -> String {
    let mut s = String::new();
    for (idx, ch) in text.chars().enumerate() {
        if idx >= max_chars {
            s.push('…');
            break;
        }
        match ch {
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            _ => s.push(ch),
        }
    }
    s
}

fn format_entry(e: &Entry) -> String {
    if e.name.is_empty() {
        format!("message: {}", preview(&e.message, 120))
    } else {
        format!("name: {} | message: {}", preview(&e.name, 24), preview(&e.message, 120))
    }
}

fn count_map(entries: &[Entry]) -> HashMap<String, (Entry, usize)> {
    let mut out = HashMap::<String, (Entry, usize)>::new();
    for e in entries {
        let key = entry_key(e);
        let item = out.entry(key).or_insert_with(|| (e.clone(), 0));
        item.1 += 1;
    }
    out
}

fn count_added_removed(old: &[Entry], new: &[Entry]) -> (Vec<(Entry, usize)>, Vec<(Entry, usize)>) {
    let old_counts = count_map(old);
    let new_counts = count_map(new);
    let mut added = Vec::new();
    let mut removed = Vec::new();

    for (key, (entry, new_count)) in &new_counts {
        let old_count = old_counts.get(key).map(|(_, c)| *c).unwrap_or(0);
        if *new_count > old_count {
            added.push((entry.clone(), *new_count - old_count));
        }
    }
    for (key, (entry, old_count)) in &old_counts {
        let new_count = new_counts.get(key).map(|(_, c)| *c).unwrap_or(0);
        if *old_count > new_count {
            removed.push((entry.clone(), *old_count - new_count));
        }
    }

    added.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| entry_key(&a.0).cmp(&entry_key(&b.0))));
    removed.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| entry_key(&a.0).cmp(&entry_key(&b.0))));
    (added, removed)
}

fn make_diff_blocks(old: &[Entry], new: &[Entry], max_blocks: usize) -> Result<(Vec<DiffBlock>, usize, usize), String> {
    let old_lines = old.iter().map(encode_line).collect::<Vec<_>>().join("\n");
    let new_lines = new.iter().map(encode_line).collect::<Vec<_>>().join("\n");
    let diff = TextDiff::from_lines(&old_lines, &new_lines);

    let mut blocks = Vec::<DiffBlock>::new();
    let mut old_pos = 0usize;
    let mut new_pos = 0usize;
    let mut total_deleted = 0usize;
    let mut total_inserted = 0usize;
    let mut current: Option<DiffBlock> = None;

    for change in diff.iter_all_changes() {
        let value = change.value().trim_end_matches('\n');
        match change.tag() {
            ChangeTag::Equal => {
                if let Some(block) = current.take() {
                    if block.deletes.len() + block.inserts.len() > 0 && blocks.len() < max_blocks {
                        blocks.push(block);
                    }
                }
                old_pos += 1;
                new_pos += 1;
            }
            ChangeTag::Delete => {
                if current.is_none() {
                    current = Some(DiffBlock {
                        old_start: old_pos,
                        new_start: new_pos,
                        deletes: Vec::new(),
                        inserts: Vec::new(),
                    });
                }
                let entry = decode_line(value)?;
                current.as_mut().unwrap().deletes.push(entry);
                total_deleted += 1;
                old_pos += 1;
            }
            ChangeTag::Insert => {
                if current.is_none() {
                    current = Some(DiffBlock {
                        old_start: old_pos,
                        new_start: new_pos,
                        deletes: Vec::new(),
                        inserts: Vec::new(),
                    });
                }
                let entry = decode_line(value)?;
                current.as_mut().unwrap().inserts.push(entry);
                total_inserted += 1;
                new_pos += 1;
            }
        }
    }
    if let Some(block) = current {
        if block.deletes.len() + block.inserts.len() > 0 && blocks.len() < max_blocks {
            blocks.push(block);
        }
    }

    Ok((blocks, total_deleted, total_inserted))
}

fn by_name(entries: &[Entry]) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for e in entries {
        if !e.name.is_empty() {
            *out.entry(e.name.clone()).or_default() += 1;
        }
    }
    out
}

fn classify_entry(e: &Entry) -> &'static str {
    if !e.name.is_empty() {
        return "named_or_merged_dialogue";
    }
    let msg = e.message.as_str();
    if msg.starts_with('\u{300c}') || msg.starts_with('\u{300e}') {
        return "quoted_without_name";
    }
    if msg.contains('\u{ff1a}')
        || msg.contains('\u{301c}')
        || msg.ends_with('\u{7de8}')
        || msg.ends_with('\u{8a71}')
    {
        return "title_or_route_label";
    }
    if msg.chars().count() <= 12 {
        return "short_ui_or_hotspot";
    }
    "prose_or_system_text"
}

fn write_report(
    old_path: &Path,
    new_path: &Path,
    report_path: &Path,
    old: &[Entry],
    new: &[Entry],
) -> Result<(), String> {
    let old_stats = stats(old);
    let new_stats = stats(new);
    let (added, removed) = count_added_removed(old, new);
    let (blocks, total_deleted, total_inserted) = make_diff_blocks(old, new, 80)?;
    let added_chars: usize = added
        .iter()
        .map(|(e, c)| e.message.chars().count() * *c)
        .sum();
    let removed_chars: usize = removed
        .iter()
        .map(|(e, c)| e.message.chars().count() * *c)
        .sum();

    let old_names = by_name(old);
    let new_names = by_name(new);
    let mut name_delta = Vec::new();
    let all_names = old_names
        .keys()
        .chain(new_names.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for name in all_names {
        let old_count = *old_names.get(&name).unwrap_or(&0);
        let new_count = *new_names.get(&name).unwrap_or(&0);
        if old_count != new_count {
            name_delta.push((name, old_count, new_count));
        }
    }
    name_delta.sort_by(|a, b| {
        let da = a.2.abs_diff(a.1);
        let db = b.2.abs_diff(b.1);
        db.cmp(&da).then_with(|| a.0.cmp(&b.0))
    });

    let mut out = String::new();
    out.push_str("# MES JSON Diff Report\n\n");
    out.push_str(&format!("- old: `{}`\n", old_path.display()));
    out.push_str(&format!("- new: `{}`\n\n", new_path.display()));

    out.push_str("## Summary\n\n");
    out.push_str("| metric | old | new | delta |\n|---|---:|---:|---:|\n");
    out.push_str(&format!(
        "| entries | {} | {} | {:+} |\n",
        old_stats.entries,
        new_stats.entries,
        new_stats.entries as isize - old_stats.entries as isize
    ));
    out.push_str(&format!(
        "| named entries | {} | {} | {:+} |\n",
        old_stats.named_entries,
        new_stats.named_entries,
        new_stats.named_entries as isize - old_stats.named_entries as isize
    ));
    out.push_str(&format!(
        "| message chars | {} | {} | {:+} |\n",
        old_stats.message_chars,
        new_stats.message_chars,
        new_stats.message_chars as isize - old_stats.message_chars as isize
    ));
    out.push_str(&format!(
        "| name chars | {} | {} | {:+} |\n",
        old_stats.name_chars,
        new_stats.name_chars,
        new_stats.name_chars as isize - old_stats.name_chars as isize
    ));
    out.push_str(&format!(
        "| unique entries | {} | {} | {:+} |\n",
        old_stats.unique_entries,
        new_stats.unique_entries,
        new_stats.unique_entries as isize - old_stats.unique_entries as isize
    ));
    out.push_str(&format!(
        "| unique messages | {} | {} | {:+} |\n",
        old_stats.unique_messages,
        new_stats.unique_messages,
        new_stats.unique_messages as isize - old_stats.unique_messages as isize
    ));
    out.push_str(&format!(
        "| empty messages | {} | {} | {:+} |\n\n",
        old_stats.empty_messages,
        new_stats.empty_messages,
        new_stats.empty_messages as isize - old_stats.empty_messages as isize
    ));

    out.push_str(&format!(
        "Sequence diff says inserted rows = {}, deleted rows = {}. Multiset diff says added chars = {}, removed chars = {}, net = {:+}.\n\n",
        total_inserted,
        total_deleted,
        added_chars,
        removed_chars,
        added_chars as isize - removed_chars as isize
    ));

    let mut category_counts = BTreeMap::<&'static str, (usize, usize)>::new();
    let mut category_examples = BTreeMap::<&'static str, Vec<(Entry, usize)>>::new();
    for (entry, count) in &added {
        let category = classify_entry(entry);
        let item = category_counts.entry(category).or_default();
        item.0 += *count;
        item.1 += entry.message.chars().count() * *count;
        let examples = category_examples.entry(category).or_default();
        if examples.len() < 20 {
            examples.push((entry.clone(), *count));
        }
    }

    out.push_str("## Added Entry Categories\n\n");
    out.push_str("| category | entries | message chars |\n|---|---:|---:|\n");
    for (category, (count, chars)) in &category_counts {
        out.push_str(&format!("| {} | {} | {} |\n", category, count, chars));
    }
    out.push('\n');
    for (category, examples) in &category_examples {
        out.push_str(&format!("### {}\n\n", category));
        for (idx, (entry, count)) in examples.iter().enumerate() {
            out.push_str(&format!("{}. x{} {}\n", idx + 1, count, format_entry(entry)));
        }
        out.push('\n');
    }

    out.push_str("## Top Added Entries\n\n");
    for (idx, (entry, count)) in added.iter().take(80).enumerate() {
        out.push_str(&format!("{}. x{} {}\n", idx + 1, count, format_entry(entry)));
    }
    if added.is_empty() {
        out.push_str("none\n");
    }
    out.push('\n');

    out.push_str("## Top Removed Entries\n\n");
    for (idx, (entry, count)) in removed.iter().take(80).enumerate() {
        out.push_str(&format!("{}. x{} {}\n", idx + 1, count, format_entry(entry)));
    }
    if removed.is_empty() {
        out.push_str("none\n");
    }
    out.push('\n');

    out.push_str("## Name Count Delta\n\n");
    for (idx, (name, old_count, new_count)) in name_delta.iter().take(80).enumerate() {
        out.push_str(&format!(
            "{}. `{}`: {} -> {} ({:+})\n",
            idx + 1,
            preview(name, 40),
            old_count,
            new_count,
            *new_count as isize - *old_count as isize
        ));
    }
    if name_delta.is_empty() {
        out.push_str("none\n");
    }
    out.push('\n');

    out.push_str("## First Sequence Diff Blocks\n\n");
    for (idx, block) in blocks.iter().enumerate() {
        out.push_str(&format!(
            "### Block {} old#{} new#{}\n\n",
            idx + 1,
            block.old_start,
            block.new_start
        ));
        if !block.deletes.is_empty() {
            out.push_str("deleted:\n");
            for e in block.deletes.iter().take(12) {
                out.push_str(&format!("- {}\n", format_entry(e)));
            }
            if block.deletes.len() > 12 {
                out.push_str(&format!("- ... {} more\n", block.deletes.len() - 12));
            }
        }
        if !block.inserts.is_empty() {
            out.push_str("inserted:\n");
            for e in block.inserts.iter().take(12) {
                out.push_str(&format!("+ {}\n", format_entry(e)));
            }
            if block.inserts.len() > 12 {
                out.push_str(&format!("+ ... {} more\n", block.inserts.len() - 12));
            }
        }
        out.push('\n');
    }

    fs::write(report_path, out).map_err(|e| format!("{}: {e}", report_path.display()))
}

fn main() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 1 && args.len() != 3 && args.len() != 4 {
        return Err(usage(&args[0]));
    }

    let old_path = args.get(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("all.orig.json"));
    let new_path = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("all.orig_new.json"));
    let report_path = args
        .get(3)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("mes_diff_report.md"));

    let old = read_entries(&old_path)?;
    let new = read_entries(&new_path)?;
    write_report(&old_path, &new_path, &report_path, &old, &new)?;

    let old_stats = stats(&old);
    let new_stats = stats(&new);
    println!("old entries={} chars={}", old_stats.entries, old_stats.message_chars);
    println!("new entries={} chars={}", new_stats.entries, new_stats.message_chars);
    println!(
        "delta entries={:+} chars={:+}",
        new_stats.entries as isize - old_stats.entries as isize,
        new_stats.message_chars as isize - old_stats.message_chars as isize
    );
    println!("report={}", report_path.display());
    Ok(())
}
