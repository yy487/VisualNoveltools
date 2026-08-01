use encoding_rs::SHIFT_JIS;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Serialize)]
struct JsonEntry {
    #[serde(rename = "_file")]
    file: String,
    #[serde(rename = "_index")]
    index: usize,
    #[serde(rename = "_line")]
    line: usize,
    #[serde(rename = "_inst_offset")]
    inst_offset: usize,
    #[serde(rename = "_offset")]
    offset: usize,
    #[serde(rename = "_size")]
    size: usize,
    #[serde(rename = "_encoding")]
    encoding: &'static str,
    #[serde(rename = "_raw_hex")]
    raw_hex: String,
    #[serde(rename = "_prefix")]
    prefix: String,
    #[serde(rename = "_type")]
    entry_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    scr_name: Option<String>,
    #[serde(rename = "_name_line", skip_serializing_if = "Option::is_none")]
    name_line: Option<usize>,
    #[serde(rename = "_name_offset", skip_serializing_if = "Option::is_none")]
    name_offset: Option<usize>,
    #[serde(rename = "_name_size", skip_serializing_if = "Option::is_none")]
    name_size: Option<usize>,
    #[serde(rename = "_scr_msg_raw")]
    scr_msg_raw: String,
    scr_msg: String,
    message: String,
}

#[derive(Debug, Default)]
struct Report {
    files: usize,
    json_files: usize,
    entries: usize,
    named_entries: usize,
    skipped_empty: usize,
    warnings: usize,
}

#[derive(Debug)]
struct LineRecord {
    line: String,
    bytes: Vec<u8>,
    start: usize,
}

#[derive(Debug)]
struct PendingName {
    name: String,
    scr_name: String,
    line: usize,
    offset: usize,
    size: usize,
}

fn help() -> &'static str {
    "usage:\n  nbook_xscript_tool extract --input <DIR_OR_XBK> [--output <DIR>]\n\nExtracts CP932 text lines inside @WIN { ... } blocks from .xbk source files.\n.xbx/.ybk/.ybx files are ignored. Output is one UTF-8 JSON file per .xbk.\nNames come from 【...】; confirmed formatting controls are removed from scr_msg/message."
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("{}", help());
        return Ok(());
    }
    if args.first().map(String::as_str) != Some("extract") {
        return Err(format!("unknown command;\n\n{}", help()));
    }

    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut index = 1usize;
    while index < args.len() {
        match args[index].as_str() {
            "--input" | "-i" => {
                index += 1;
                input = Some(next_arg(&args, index, "--input")?.into());
            }
            "--output" | "-o" => {
                index += 1;
                output = Some(next_arg(&args, index, "--output")?.into());
            }
            value if !value.starts_with('-') && input.is_none() => input = Some(value.into()),
            value if !value.starts_with('-') && output.is_none() => output = Some(value.into()),
            other => return Err(format!("unknown option: {other}\n\n{}", help())),
        }
        index += 1;
    }

    let input = input.ok_or_else(|| format!("--input is required\n\n{}", help()))?;
    if !input.exists() {
        return Err(format!("input does not exist: {}", input.display()));
    }
    let output = output.unwrap_or_else(|| default_output(&input));
    if output.exists() {
        return Err(format!(
            "refusing to overwrite existing output directory: {}",
            output.display()
        ));
    }

    let files = collect_xbk_files(&input)?;
    if files.is_empty() {
        return Err(format!("no .xbk files found under {}", input.display()));
    }
    fs::create_dir_all(&output)
        .map_err(|error| format!("failed to create {}: {error}", output.display()))?;

    let mut report = Report::default();
    for (path, relative) in files {
        report.files += 1;
        let (entries, warnings, names, skipped_empty) = extract_file(&path, &relative)?;
        let json_path = output.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        let json_path = json_path.with_extension("json");
        if let Some(parent) = json_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(&entries)
            .map_err(|error| format!("failed to serialize {}: {error}", json_path.display()))?;
        fs::write(&json_path, format!("{json}\n"))
            .map_err(|error| format!("failed to write {}: {error}", json_path.display()))?;
        report.json_files += 1;
        report.entries += entries.len();
        report.named_entries += names;
        report.skipped_empty += skipped_empty;
        report.warnings += warnings;
        println!(
            "[extract] file={} entries={} named_entries={} skipped_empty={} warnings={}",
            relative,
            entries.len(),
            names,
            skipped_empty,
            warnings
        );
    }
    println!(
        "[extract] scanned_files={} json_files={} extracted_entries={} named_entries={} skipped_empty={} warnings={} output={}",
        report.files,
        report.json_files,
        report.entries,
        report.named_entries,
        report.skipped_empty,
        report.warnings,
        output.display()
    );
    Ok(())
}

fn next_arg<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("{option} requires a value"))
}

fn default_output(input: &Path) -> PathBuf {
    let base = if input.is_dir() {
        input.to_path_buf()
    } else {
        input
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    };
    let stem = if input.is_dir() {
        input
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| "input".to_string())
    } else {
        input
            .file_stem()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| "input".to_string())
    };
    base.join(format!("{stem}_json"))
}

fn collect_xbk_files(input: &Path) -> Result<Vec<(PathBuf, String)>> {
    let base = if input.is_dir() {
        input.to_path_buf()
    } else {
        input
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    };
    let mut files = Vec::new();
    if input.is_file() {
        if !is_ext(input, "xbk") {
            return Err(format!("input file is not .xbk: {}", input.display()));
        }
        files.push((input.to_path_buf(), file_name(input)?));
        return Ok(files);
    }
    collect_xbk_recursive(&base, input, &mut files)?;
    files.sort_by(|left, right| left.1.cmp(&right.1));
    Ok(files)
}

fn collect_xbk_recursive(
    base: &Path,
    current: &Path,
    out: &mut Vec<(PathBuf, String)>,
) -> Result<()> {
    let mut children = fs::read_dir(current)
        .map_err(|error| format!("failed to read {}: {error}", current.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read directory entry: {error}"))?;
    children.sort_by_key(|entry| entry.path());
    for child in children {
        let path = child.path();
        let metadata = child
            .metadata()
            .map_err(|error| format!("failed to stat {}: {error}", path.display()))?;
        if metadata.is_dir() {
            collect_xbk_recursive(base, &path, out)?;
        } else if metadata.is_file() && is_ext(&path, "xbk") {
            let relative = path
                .strip_prefix(base)
                .map_err(|error| format!("failed to make relative path: {error}"))?
                .to_string_lossy()
                .replace('\\', "/");
            out.push((path, relative));
        }
    }
    Ok(())
}

fn is_ext(path: &Path, expected: &str) -> bool {
    path.extension()
        .is_some_and(|value| value.eq_ignore_ascii_case(expected))
}

fn file_name(path: &Path) -> Result<String> {
    path.file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .ok_or_else(|| format!("invalid file path: {}", path.display()))
}

fn decode_lines(path: &Path) -> Result<Vec<LineRecord>> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let mut records = Vec::new();
    let mut start = 0usize;
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] != b'\r' && bytes[cursor] != b'\n' {
            cursor += 1;
            continue;
        }
        let newline_len =
            if bytes[cursor] == b'\r' && cursor + 1 < bytes.len() && bytes[cursor + 1] == b'\n' {
                2
            } else {
                1
            };
        let raw = &bytes[start..cursor];
        let (text, _, had_errors) = SHIFT_JIS.decode(raw);
        if had_errors {
            return Err(format!(
                "{} line at byte 0x{start:X} is not valid CP932",
                path.display()
            ));
        }
        let text = text.into_owned();
        let (roundtrip, _, encode_errors) = SHIFT_JIS.encode(&text);
        if encode_errors || roundtrip.as_ref() != raw {
            return Err(format!(
                "{} line at byte 0x{start:X} failed CP932 byte round-trip",
                path.display()
            ));
        }
        records.push(LineRecord {
            line: text,
            bytes: raw.to_vec(),
            start,
        });
        cursor += newline_len;
        start = cursor;
    }
    if start < bytes.len() {
        let raw = &bytes[start..];
        let (text, _, had_errors) = SHIFT_JIS.decode(raw);
        if had_errors {
            return Err(format!(
                "{} final line at byte 0x{start:X} is not valid CP932",
                path.display()
            ));
        }
        let text = text.into_owned();
        let (roundtrip, _, encode_errors) = SHIFT_JIS.encode(&text);
        if encode_errors || roundtrip.as_ref() != raw {
            return Err(format!(
                "{} final line failed CP932 byte round-trip",
                path.display()
            ));
        }
        records.push(LineRecord {
            line: text,
            bytes: raw.to_vec(),
            start,
        });
    }
    Ok(records)
}

fn ascii_trim(text: &str) -> &str {
    text.trim_matches([' ', '\t'])
}

fn ascii_trim_start(text: &str) -> &str {
    text.trim_start_matches([' ', '\t'])
}

fn filter_controls(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0usize;
    while index < chars.len() {
        let ch = chars[index];
        if ch == '@' || ch == '_' || ch == '*' {
            let valid = if ch == '*' {
                find_control_end(&chars, index, ch).filter(|end| {
                    let token = &chars[index + 1..*end];
                    token.len() == 6 && token.iter().all(|value| value.is_ascii_hexdigit())
                })
            } else {
                find_control_end(&chars, index, ch).filter(|end| {
                    let token = &chars[index + 1..*end];
                    !token.is_empty() && token.iter().all(|value| value.is_ascii_alphanumeric())
                })
            };
            if let Some(end) = valid {
                index = end + 1;
                continue;
            }
        }
        if matches!(ch, '#' | '<' | '>' | '+') {
            index += 1;
            continue;
        }
        out.push(ch);
        index += 1;
    }
    out.trim_matches([' ', '\t', '　']).to_string()
}

fn find_control_end(chars: &[char], start: usize, delimiter: char) -> Option<usize> {
    chars[start + 1..]
        .iter()
        .position(|value| *value == delimiter)
        .map(|offset| start + 1 + offset)
}

fn split_name_prefix(text: &str) -> Option<(&str, &str)> {
    if !text.starts_with('【') {
        return None;
    }
    let end = text.find('】')?;
    let end_marker = end + '】'.len_utf8();
    Some((&text['【'.len_utf8()..end], &text[end_marker..]))
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>()
}

fn cp932_bytes(label: &str, text: &str) -> Result<Vec<u8>> {
    let (bytes, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        return Err(format!(
            "{label} contains characters that cannot be encoded as CP932"
        ));
    }
    Ok(bytes.into_owned())
}

fn extract_file(path: &Path, relative: &str) -> Result<(Vec<JsonEntry>, usize, usize, usize)> {
    let lines = decode_lines(path)?;
    let mut entries = Vec::new();
    let mut warnings = 0usize;
    let mut named_entries = 0usize;
    let mut skipped_empty = 0usize;
    let mut in_window = false;
    let mut pending: Option<PendingName> = None;
    for (line_index, record) in lines.iter().enumerate() {
        let line_number = line_index + 1;
        let trimmed = ascii_trim(&record.line);
        if trimmed == "@WIN {" {
            in_window = true;
            continue;
        }
        if in_window && trimmed == "}" {
            if pending.take().is_some() {
                warnings += 1;
                eprintln!("[extract][warning] {relative}:{line_number}: name candidate has no following text");
            }
            in_window = false;
            continue;
        }
        if !in_window || trimmed.is_empty() {
            continue;
        }

        let body = ascii_trim_start(&record.line);
        let indent_len = record.line.len() - body.len();
        let mut message_start = indent_len;
        let mut raw_message = body.to_string();
        let mut clean_message = filter_controls(body);
        let mut name = pending.as_ref().map(|value| value.name.clone());
        let mut scr_name = pending.as_ref().map(|value| value.scr_name.clone());
        let mut name_line = pending.as_ref().map(|value| value.line);
        let mut name_offset = pending.as_ref().map(|value| value.offset);
        let mut name_size = pending.as_ref().map(|value| value.size);

        if let Some((raw_name, suffix)) = split_name_prefix(body) {
            let name_marker_end = body.find('】').unwrap() + '】'.len_utf8();
            let name_prefix = &body[..name_marker_end];
            let name_prefix_bytes =
                cp932_bytes(&format!("{relative}:{line_number} name"), name_prefix)?;
            let name_body_bytes = &record.bytes[indent_len..indent_len + name_prefix_bytes.len()];
            let pending_name = PendingName {
                name: filter_controls(raw_name),
                scr_name: raw_name.to_string(),
                line: line_number,
                offset: record.start + indent_len,
                size: name_body_bytes.len(),
            };
            name = Some(pending_name.name.clone());
            scr_name = Some(pending_name.scr_name.clone());
            name_line = Some(pending_name.line);
            name_offset = Some(pending_name.offset);
            name_size = Some(pending_name.size);
            message_start = indent_len + name_prefix.len();
            raw_message = suffix.to_string();
            clean_message = filter_controls(suffix);
            if clean_message.is_empty() {
                if pending.is_some() {
                    warnings += 1;
                    eprintln!(
                        "[extract][warning] {relative}:{line_number}: consecutive name markers"
                    );
                }
                pending = Some(pending_name);
                continue;
            }
            pending = None;
        } else if !clean_message.is_empty() {
            pending = None;
        }

        if clean_message.is_empty() {
            skipped_empty += 1;
            continue;
        }

        if !record.line.starts_with("\t\t") {
            warnings += 1;
            eprintln!("[extract][warning] {relative}:{line_number}: text line lacks the usual 0D0A0909 marker");
        }
        let message_prefix = &record.line[..message_start];
        let message_prefix_bytes =
            cp932_bytes(&format!("{relative}:{line_number} prefix"), message_prefix)?;
        let body_bytes = &record.bytes[message_prefix_bytes.len()..];
        let entry = JsonEntry {
            file: relative.to_string(),
            index: entries.len(),
            line: line_number,
            inst_offset: record.start,
            offset: record.start + message_prefix_bytes.len(),
            size: body_bytes.len(),
            encoding: "cp932",
            raw_hex: hex(body_bytes),
            prefix: message_prefix.to_string(),
            entry_type: "dialogue",
            name,
            scr_name,
            name_line,
            name_offset,
            name_size,
            scr_msg_raw: raw_message,
            scr_msg: clean_message.clone(),
            message: clean_message,
        };
        if entry.name.is_some() {
            named_entries += 1;
        }
        entries.push(entry);
    }
    if in_window {
        warnings += 1;
        eprintln!("[extract][warning] {relative}: unterminated @WIN block");
    }
    Ok((entries, warnings, named_entries, skipped_empty))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_confirmed_controls() {
        assert_eq!(filter_controls("う@80@@80@><"), "う");
        assert_eq!(filter_controls("_3_……。_0_#"), "……。");
        assert_eq!(filter_controls("*FFFF00*水原*FFFFFF*"), "水原");
        assert_eq!(filter_controls("<　+"), "");
    }

    #[test]
    fn splits_name_prefix() {
        assert_eq!(
            split_name_prefix("【藍澤　清】<　+"),
            Some(("藍澤　清", "<　+"))
        );
        assert_eq!(
            split_name_prefix("【水原】先生ですか？#"),
            Some(("水原", "先生ですか？#"))
        );
    }

    #[test]
    fn trims_only_ascii_indentation() {
        assert_eq!(ascii_trim_start("\t\t　本文"), "　本文");
    }
}
