use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Entry {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    message: String,
}

fn speakers() -> &'static [&'static str] {
    &[
        "若葉＆宏美＆祥子",
        "竜一＆宏美＆祥子",
        "宏美＆祥子",
        "若葉＆宏美",
        "穣＆若葉",
        "竜一＆高志",
        "宏美＆高志",
        "竜一＆宏美",
        "警官達",
        "ババァ",
        "とも子",
        "あかり",
        "竜一",
        "高志",
        "若葉",
        "宏美",
        "祥子",
        "穣",
        "恵美",
        "伊藤",
        "大石",
        "祐司",
        "親父",
        "義昌",
        "猫",
    ]
}

fn junk_fragments() -> &'static [&'static str] {
    &[
        "ﾖ", "ョ", "眤", "狢", "燿", "ｦ", "ﾆ", "ﾞ",
    ]
}

fn split_inline_speaker(message: &str) -> Option<(&'static str, String)> {
    let trimmed = message.trim_start_matches(|ch: char| ch.is_whitespace() || ch == '\u{3000}');
    for &speaker in speakers() {
        if let Some(rest) = trimmed.strip_prefix(speaker) {
            if rest.starts_with('「') || rest.starts_with('『') {
                return Some((speaker, rest.to_string()));
            }
        }
    }
    None
}

fn is_junk_fragment(message: &str) -> bool {
    junk_fragments().contains(&message)
}

fn usage(program: &str) -> String {
    format!("usage: {program} <input.json> <output.json>")
}

fn main() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        return Err(usage(&args[0]));
    }
    let input = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);
    let text = fs::read_to_string(&input).map_err(|e| format!("{}: {e}", input.display()))?;
    let entries: Vec<Entry> =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", input.display()))?;

    let mut out = Vec::with_capacity(entries.len());
    let mut split_count = 0usize;
    let mut junk_count = 0usize;
    for mut entry in entries {
        if entry.name.is_none() {
            if is_junk_fragment(&entry.message) {
                junk_count += 1;
                continue;
            }
            if let Some((speaker, message)) = split_inline_speaker(&entry.message) {
                entry.name = Some(speaker.to_string());
                entry.message = message;
                split_count += 1;
            }
        }
        out.push(entry);
    }

    let json = serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?;
    fs::write(&output, json).map_err(|e| format!("{}: {e}", output.display()))?;
    println!(
        "input={} output={} entries={} split_inline={} removed_junk={}",
        input.display(),
        output.display(),
        out.len(),
        split_count,
        junk_count
    );
    Ok(())
}
