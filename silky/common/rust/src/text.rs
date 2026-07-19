use crate::codec::encode_text;
use crate::mes::{Command, MesScript};
use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::HashSet;

const NAME_BLOCK_PUSH_VALUES: &[i64] = &[83_886_080, 117_440_512, 167_772_160];

#[derive(Clone, Debug)]
enum RubySlotKind {
    Reading,
    Separator,
}

#[derive(Clone, Debug)]
enum TextPart {
    Text {
        node: usize,
        text: String,
    },
    Ruby {
        base_node: usize,
        base: String,
        reading_slots: Vec<(usize, String, RubySlotKind)>,
    },
    Newline,
}

#[derive(Clone, Debug)]
struct TextBlock {
    parts: Vec<TextPart>,
    end_command: usize,
    name: Option<String>,
    name_node: Option<usize>,
    valid: bool,
}

#[derive(Debug, Serialize)]
pub struct TextEntry {
    #[serde(rename = "_file")]
    file: String,
    #[serde(rename = "_index")]
    index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    source_name: Option<String>,
    scr_msg: String,
    message: String,
    #[serde(rename = "_inst_offset")]
    instruction_offset: usize,
    #[serde(rename = "_type")]
    entry_type: &'static str,
    #[serde(rename = "_opcode")]
    opcode: String,
    #[serde(rename = "_encoding")]
    encoding: String,
    #[serde(rename = "_policy")]
    policy: &'static str,
}

#[derive(Debug)]
pub struct ExtractResult {
    pub entries: Vec<TextEntry>,
    pub warnings: Vec<String>,
    pub skipped_blocks: usize,
}

#[derive(Debug, Default)]
pub struct InjectionStats {
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub warnings: Vec<String>,
}

pub fn extract_entries(script: &MesScript, source_file: &str) -> ExtractResult {
    let blocks = text_blocks(script);
    let mut entries = Vec::new();
    let mut warnings = script.warnings.clone();
    let mut skipped_blocks = 0usize;

    for block in blocks {
        if !block.valid {
            skipped_blocks += 1;
            let offset = first_part_node(&block)
                .and_then(|node| script.nodes.get(node))
                .map_or(0, |node| node.old_offset);
            warnings.push(format!(
                "text block at code offset 0x{offset:08X} contains an undecodable or malformed ruby string; block skipped"
            ));
            continue;
        }
        let scr_msg = join_parts(&block.parts);
        if scr_msg.is_empty() {
            continue;
        }
        let Some(first_node) = first_part_node(&block) else {
            continue;
        };
        let node = &script.nodes[first_node];
        let opcode = script
            .command(first_node)
            .map(command_label)
            .unwrap_or_else(|| "RAW".to_owned());
        let name = block.name.clone();
        let entry_type = if name.is_some() {
            "dialogue"
        } else {
            "monologue"
        };
        let index = entries.len();
        entries.push(TextEntry {
            file: source_file.to_owned(),
            index,
            name: name.clone(),
            source_name: name,
            scr_msg: scr_msg.clone(),
            message: scr_msg,
            instruction_offset: script.header_size() + node.old_offset as usize,
            entry_type,
            opcode,
            encoding: script.encoding.clone(),
            policy: "relocate",
        });
    }

    ExtractResult {
        entries,
        warnings,
        skipped_blocks,
    }
}

pub fn inject_entries(
    script: &mut MesScript,
    json_text: &str,
    source_file: &str,
) -> Result<InjectionStats> {
    let value: Value = serde_json::from_str(json_text).context("invalid translation JSON")?;
    let entries = match &value {
        Value::Array(entries) => entries,
        Value::Object(root) => root
            .get("entries")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                anyhow!("JSON root must be an array or an object containing an entries array")
            })?,
        _ => bail!("JSON root must be an array or an object containing an entries array"),
    };

    let all_blocks = text_blocks(script);
    if let Some(block) = all_blocks.iter().find(|block| !block.valid) {
        let offset = first_part_node(block)
            .and_then(|node| script.nodes.get(node))
            .map_or(0, |node| node.old_offset);
        bail!(
            "MES contains an undecodable or unsupported ruby/text block at code offset 0x{offset:08X}; refusing partial injection"
        );
    }
    let blocks: Vec<TextBlock> = all_blocks
        .into_iter()
        .filter(|block| block.valid && !join_parts(&block.parts).is_empty())
        .collect();
    if entries.len() != blocks.len() {
        bail!(
            "JSON entry count does not match MES text blocks: json={}, mes={}",
            entries.len(),
            blocks.len()
        );
    }

    #[derive(Debug)]
    struct Plan {
        block_index: usize,
        message: String,
        name: Option<String>,
        changed: bool,
    }

    let mut plans = Vec::with_capacity(entries.len());
    let mut used_indices = HashSet::new();
    let mut warnings = script.warnings.clone();

    for (fallback_index, value) in entries.iter().enumerate() {
        let object = value
            .as_object()
            .ok_or_else(|| anyhow!("JSON entry {fallback_index} is not an object"))?;
        let index = read_index(object, fallback_index)?;
        if !used_indices.insert(index) {
            bail!("duplicate JSON _index: {index}");
        }
        let block = blocks
            .get(index)
            .ok_or_else(|| anyhow!("JSON _index is out of range: {index}"))?;

        if let Some(file) = object.get("_file") {
            let file = file
                .as_str()
                .ok_or_else(|| anyhow!("entry index={index}: _file must be a string"))?;
            if !file.is_empty() && !file.eq_ignore_ascii_case(source_file) {
                bail!("entry index={index}: _file mismatch, json={file:?}, source={source_file:?}");
            }
        }

        let scr_msg = required_string(object, "scr_msg", index)?;
        let message = required_string(object, "message", index)?;
        reject_illegal_text(message, "message", index)?;
        let old_scr = join_parts(&block.parts);
        if scr_msg != old_scr {
            bail!("entry index={index}: scr_msg mismatch, json={scr_msg:?}, source={old_scr:?}");
        }
        let expected_newlines = block
            .parts
            .iter()
            .filter(|part| matches!(part, TextPart::Newline))
            .count();
        let actual_newlines = message.match_indices("\\n").count();
        if actual_newlines != expected_newlines {
            bail!(
                "entry index={index}: literal \\n control count changed: expected={expected_newlines}, actual={actual_newlines}"
            );
        }

        let old_name = block.name.as_deref();
        if let Some(source_name) = object.get("_scr_name") {
            let source_name = source_name
                .as_str()
                .ok_or_else(|| anyhow!("entry index={index}: _scr_name must be a string"))?;
            if Some(source_name) != old_name {
                bail!(
                    "entry index={index}: _scr_name mismatch, json={source_name:?}, source={old_name:?}"
                );
            }
        }

        let name = match object.get("name") {
            Some(value) => {
                let value = value
                    .as_str()
                    .ok_or_else(|| anyhow!("entry index={index}: name must be a string"))?;
                reject_illegal_text(value, "name", index)?;
                if old_name.is_none() || block.name_node.is_none() {
                    bail!("entry index={index}: JSON contains name but source block has no writable name");
                }
                if !object.contains_key("_scr_name") {
                    warnings.push(format!(
                        "entry index={index}: legacy name accepted without _scr_name source validation"
                    ));
                }
                Some(value.to_owned())
            }
            None => None,
        };

        validate_encoding(message, block, script, index)?;
        if let Some(name) = &name {
            encode_text(name, &script.encoding)
                .with_context(|| format!("entry index={index}: name encoding failed"))?;
        }
        // Project policy verified by the original Python tool: every injection
        // blanks ruby readings, even when message/name are otherwise unchanged.
        let changed = message != old_scr
            || name.as_deref().is_some_and(|name| Some(name) != old_name)
            || has_ruby_reading(block);
        plans.push(Plan {
            block_index: index,
            message: message.to_owned(),
            name,
            changed,
        });
    }

    plans.sort_by_key(|plan| plan.block_index);
    let mut stats = InjectionStats {
        json_entries: entries.len(),
        warnings,
        ..InjectionStats::default()
    };
    for plan in plans {
        if !plan.changed {
            stats.unchanged += 1;
            continue;
        }
        apply_plan(
            script,
            &blocks[plan.block_index],
            &plan.message,
            plan.name.as_deref(),
        )?;
        stats.patched += 1;
    }
    Ok(stats)
}

fn text_blocks(script: &MesScript) -> Vec<TextBlock> {
    let commands = script.command_node_indices();
    let mut result = Vec::new();
    let mut position = 0usize;
    while position < commands.len() {
        let opcode = command_at(script, &commands, position).map(|command| command.opcode);
        let start = if opcode == Some(0x19) {
            position + 1
        } else if matches!(opcode, Some(0x0a | 0x0b | 0x1c)) {
            position
        } else {
            position += 1;
            continue;
        };
        let block = collect_block(script, &commands, start);
        position = block.end_command.max(position + 1);
        if !block.parts.is_empty() || !block.valid {
            result.push(block);
        }
    }
    result
}

fn collect_block(script: &MesScript, commands: &[usize], start: usize) -> TextBlock {
    let mut parts = Vec::new();
    let mut name = None;
    let mut name_node = None;
    let mut valid = true;
    let mut position = start;

    while position < commands.len() {
        if let Some((detected, node)) = detect_name(script, commands, position) {
            name = Some(detected);
            name_node = Some(node);
            position += 1;
            continue;
        }
        let Some(command) = command_at(script, commands, position) else {
            position += 1;
            continue;
        };
        match command.opcode {
            0x1c => match command.first_integer() {
                Some(1) => match match_ruby(script, commands, position) {
                    Some((part, end)) => {
                        parts.push(part);
                        position = end;
                        continue;
                    }
                    None => valid = false,
                },
                Some(0) => parts.push(TextPart::Newline),
                _ => {}
            },
            0x0a | 0x0b => {
                let node = commands[position];
                match command.first_string().and_then(|value| value.text()) {
                    Some(text) => parts.push(TextPart::Text {
                        node,
                        text: text.to_owned(),
                    }),
                    None => valid = false,
                }
            }
            0x19 | 0x14 | 0x15 | 0x16 | 0x1a | 0x1b => break,
            _ => {}
        }
        position += 1;
    }

    TextBlock {
        parts,
        end_command: position,
        name,
        name_node,
        valid,
    }
}

fn detect_name(script: &MesScript, commands: &[usize], position: usize) -> Option<(String, usize)> {
    let first = command_at(script, commands, position)?;
    if first.opcode != 0x33 {
        return None;
    }
    let name = first.first_string()?.text()?;
    if name.is_ascii() {
        return None;
    }
    let second = command_at(script, commands, position + 1)?;
    if second.opcode != 0x32 || !NAME_BLOCK_PUSH_VALUES.contains(&second.first_integer()?) {
        return None;
    }

    let pattern_a = command_at(script, commands, position + 2)?.opcode == 0x32
        && command_at(script, commands, position + 3)?.opcode == 0x18;
    let pattern_b = command_at(script, commands, position + 2)?.opcode == 0x32
        && command_at(script, commands, position + 3)?.opcode == 0x34
        && command_at(script, commands, position + 4)?.opcode == 0x32
        && command_at(script, commands, position + 5)?.opcode == 0x18;
    (pattern_a || pattern_b).then(|| (name.to_owned(), commands[position]))
}

fn match_ruby(
    script: &MesScript,
    commands: &[usize],
    position: usize,
) -> Option<(TextPart, usize)> {
    let tns = command_at(script, commands, position)?;
    if tns.opcode != 0x1c || tns.first_integer() != Some(1) {
        return None;
    }
    let mut cursor = position + 1;
    let mut slots = Vec::new();
    while cursor < commands.len() {
        let command = command_at(script, commands, cursor)?;
        if command.opcode == 0x00 {
            break;
        }
        if !matches!(command.opcode, 0x0a | 0x0b) {
            return None;
        }
        let text = command.first_string()?.text()?.to_owned();
        let kind = if command.opcode == 0x0b || text == "　" {
            RubySlotKind::Separator
        } else {
            RubySlotKind::Reading
        };
        slots.push((commands[cursor], text, kind));
        cursor += 1;
    }
    if slots.is_empty() || command_at(script, commands, cursor)?.opcode != 0x00 {
        return None;
    }
    cursor += 1;
    let base_command = command_at(script, commands, cursor)?;
    if base_command.opcode != 0x0a {
        return None;
    }
    let base = base_command.first_string()?.text()?.to_owned();
    Some((
        TextPart::Ruby {
            base_node: commands[cursor],
            base,
            reading_slots: slots,
        },
        cursor + 1,
    ))
}

fn command_at<'a>(
    script: &'a MesScript,
    commands: &[usize],
    position: usize,
) -> Option<&'a Command> {
    script.command(*commands.get(position)?)
}

fn join_parts(parts: &[TextPart]) -> String {
    let mut output = String::new();
    for part in parts {
        match part {
            TextPart::Text { text, .. } => output.push_str(text),
            TextPart::Ruby { base, .. } => output.push_str(base),
            TextPart::Newline => output.push_str("\\n"),
        }
    }
    output
}

fn first_part_node(block: &TextBlock) -> Option<usize> {
    block.parts.iter().find_map(|part| match part {
        TextPart::Text { node, .. } => Some(*node),
        TextPart::Ruby { base_node, .. } => Some(*base_node),
        TextPart::Newline => None,
    })
}

fn has_ruby_reading(block: &TextBlock) -> bool {
    block.parts.iter().any(|part| match part {
        TextPart::Ruby { reading_slots, .. } => reading_slots
            .iter()
            .any(|(_, _, kind)| matches!(kind, RubySlotKind::Reading)),
        _ => false,
    })
}

fn command_label(command: &Command) -> String {
    if command.name().is_empty() {
        format!("0x{:02X}", command.opcode)
    } else {
        command.name().to_owned()
    }
}

fn read_index(object: &Map<String, Value>, fallback: usize) -> Result<usize> {
    match object.get("_index") {
        None => Ok(fallback),
        Some(Value::Number(number)) => number
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| anyhow!("entry {fallback}: _index must be a non-negative integer")),
        Some(_) => bail!("entry {fallback}: _index must be a non-negative integer"),
    }
}

fn required_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    index: usize,
) -> Result<&'a str> {
    object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("entry index={index}: missing or non-string {field}"))
}

fn reject_illegal_text(text: &str, field: &str, index: usize) -> Result<()> {
    if text.contains('\0') {
        bail!("entry index={index}: {field} contains NUL");
    }
    if text.contains(['\r', '\n']) {
        bail!(
            "entry index={index}: {field} contains a real CR/LF; preserve the script's literal \\n controls instead"
        );
    }
    Ok(())
}

fn validate_encoding(
    message: &str,
    block: &TextBlock,
    script: &MesScript,
    index: usize,
) -> Result<()> {
    let segments: Vec<&str> = message.split("\\n").collect();
    let groups = part_groups(&block.parts);
    if segments.len() != groups.len() {
        bail!(
            "entry index={index}: message segment count does not match source controls: {} != {}",
            segments.len(),
            groups.len()
        );
    }
    for (segment_index, segment) in segments.iter().enumerate() {
        encode_text(segment, &script.encoding).with_context(|| {
            format!("entry index={index}, segment={segment_index}: message encoding failed")
        })?;
    }
    Ok(())
}

fn part_groups(parts: &[TextPart]) -> Vec<Vec<&TextPart>> {
    let mut groups = vec![Vec::new()];
    for part in parts {
        if matches!(part, TextPart::Newline) {
            groups.push(Vec::new());
        } else {
            groups.last_mut().expect("initial group exists").push(part);
        }
    }
    groups
        .into_iter()
        .filter(|group| !group.is_empty())
        .collect()
}

fn apply_plan(
    script: &mut MesScript,
    block: &TextBlock,
    message: &str,
    name: Option<&str>,
) -> Result<()> {
    if let (Some(name), Some(node)) = (name, block.name_node) {
        set_string(script, node, name.to_owned())?;
    }

    let groups = part_groups(&block.parts);
    let segments: Vec<&str> = message.split("\\n").collect();
    if groups.len() != segments.len() {
        bail!("internal error: translation group count changed after validation");
    }

    for (group, segment) in groups.into_iter().zip(segments) {
        let mut assigned = false;
        for part in group {
            match part {
                TextPart::Text { node, .. } => {
                    set_string(
                        script,
                        *node,
                        if assigned {
                            String::new()
                        } else {
                            segment.to_owned()
                        },
                    )?;
                    assigned = true;
                }
                TextPart::Ruby {
                    base_node,
                    reading_slots,
                    ..
                } => {
                    for (node, original, kind) in reading_slots {
                        if matches!(kind, RubySlotKind::Reading) {
                            let count = original.chars().filter(|ch| *ch != '　').count();
                            set_string(script, *node, "　".repeat(count))?;
                        }
                    }
                    set_string(
                        script,
                        *base_node,
                        if assigned {
                            String::new()
                        } else {
                            segment.to_owned()
                        },
                    )?;
                    assigned = true;
                }
                TextPart::Newline => unreachable!("newlines are group separators"),
            }
        }
    }
    Ok(())
}

fn set_string(script: &mut MesScript, node: usize, value: String) -> Result<()> {
    script
        .command_mut(node)
        .and_then(Command::first_string_mut)
        .ok_or_else(|| anyhow!("text slot no longer points to a string command"))?
        .set_text(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode_text;
    use crate::mes::encode_script_string;

    fn fixture() -> Vec<u8> {
        let name = encode_text("リルカ", "cp932").unwrap();
        let body = encode_text("本文", "cp932").unwrap();
        let mut code = Vec::new();
        code.push(0x19);
        code.extend_from_slice(&0u32.to_be_bytes());
        code.push(0x33);
        code.extend_from_slice(&name);
        code.push(0);
        code.push(0x32);
        code.extend_from_slice(&117_440_512i32.to_le_bytes());
        code.push(0x32);
        code.extend_from_slice(&0i32.to_le_bytes());
        code.push(0x18);
        code.push(0x0b);
        code.extend_from_slice(&body);
        code.push(0);
        code.push(0x00);
        let mut file = Vec::new();
        file.extend_from_slice(&1u32.to_le_bytes());
        file.extend_from_slice(&0u32.to_le_bytes());
        file.extend_from_slice(&0u32.to_le_bytes());
        file.extend_from_slice(&code);
        file
    }

    fn ruby_fixture() -> Vec<u8> {
        let reading = encode_script_string(0x0a, "よみ", "cp932").unwrap();
        let separator = encode_text("　", "cp932").unwrap();
        let base = encode_script_string(0x0a, "読み", "cp932").unwrap();
        let next = encode_text("次", "cp932").unwrap();
        let mut code = Vec::new();
        code.push(0x19);
        code.extend_from_slice(&0u32.to_be_bytes());
        code.extend_from_slice(&[0x1c, 1]);
        code.push(0x0a);
        code.extend_from_slice(&reading);
        code.push(0);
        code.push(0x0b);
        code.extend_from_slice(&separator);
        code.push(0);
        code.push(0x00);
        code.push(0x0a);
        code.extend_from_slice(&base);
        code.push(0);
        code.extend_from_slice(&[0x1c, 0]);
        code.push(0x0b);
        code.extend_from_slice(&next);
        code.push(0);
        code.push(0x00);
        let mut file = Vec::new();
        file.extend_from_slice(&1u32.to_le_bytes());
        file.extend_from_slice(&0u32.to_le_bytes());
        file.extend_from_slice(&0u32.to_le_bytes());
        file.extend_from_slice(&code);
        file
    }

    #[test]
    fn extraction_uses_project_json_contract() {
        let script = MesScript::parse(&fixture(), "cp932").unwrap();
        let result = extract_entries(&script, "TEST.MES");
        assert_eq!(result.entries.len(), 1);
        let json = serde_json::to_value(&result.entries).unwrap();
        let entry = &json[0];
        assert_eq!(entry["name"], "リルカ");
        assert_eq!(entry["_scr_name"], "リルカ");
        assert_eq!(entry["scr_msg"], "本文");
        assert_eq!(entry["message"], "本文");
    }

    #[test]
    fn unchanged_injection_is_byte_exact() {
        let source = fixture();
        let mut script = MesScript::parse(&source, "cp932").unwrap();
        let extracted = extract_entries(&script, "TEST.MES");
        let json = serde_json::to_string(&extracted.entries).unwrap();
        let stats = inject_entries(&mut script, &json, "TEST.MES").unwrap();
        assert_eq!(stats.patched, 0);
        assert_eq!(stats.unchanged, 1);
        assert_eq!(script.to_bytes().unwrap(), source);
    }

    #[test]
    fn changed_body_and_name_are_reextractable() {
        let source = fixture();
        let mut script = MesScript::parse(&source, "cp932").unwrap();
        let extracted = extract_entries(&script, "TEST.MES");
        let mut json = serde_json::to_value(&extracted.entries).unwrap();
        json[0]["message"] = Value::String("変更後".to_owned());
        json[0]["name"] = Value::String("新名".to_owned());
        let stats = inject_entries(&mut script, &json.to_string(), "TEST.MES").unwrap();
        assert_eq!(stats.patched, 1);
        let rebuilt = script.to_bytes().unwrap();
        let reparsed = MesScript::parse(&rebuilt, "cp932").unwrap();
        let result = extract_entries(&reparsed, "TEST.MES");
        let value = serde_json::to_value(&result.entries).unwrap();
        assert_eq!(value[0]["name"], "新名");
        assert_eq!(value[0]["message"], "変更後");
    }

    #[test]
    fn source_mismatch_is_fatal_and_does_not_mutate() {
        let source = fixture();
        let mut script = MesScript::parse(&source, "cp932").unwrap();
        let before = script.to_bytes().unwrap();
        let extracted = extract_entries(&script, "TEST.MES");
        let mut json = serde_json::to_value(&extracted.entries).unwrap();
        json[0]["scr_msg"] = Value::String("改坏原文".to_owned());
        assert!(inject_entries(&mut script, &json.to_string(), "TEST.MES").is_err());
        assert_eq!(script.to_bytes().unwrap(), before);
    }

    #[test]
    fn ruby_is_blank_on_every_injection_and_literal_newline_is_preserved() {
        let source = ruby_fixture();
        let mut script = MesScript::parse(&source, "cp932").unwrap();
        let extracted = extract_entries(&script, "RUBY.MES");
        let mut json = serde_json::to_value(&extracted.entries).unwrap();
        assert_eq!(json[0]["scr_msg"], "読み\\n次");

        let unchanged = serde_json::to_string(&json).unwrap();
        let stats = inject_entries(&mut script, &unchanged, "RUBY.MES").unwrap();
        assert_eq!(stats.patched, 1);
        let blanked = script.to_bytes().unwrap();
        assert_ne!(blanked, source);
        let blanked_script = MesScript::parse(&blanked, "cp932").unwrap();
        let reading = blanked_script
            .nodes
            .iter()
            .filter_map(|node| match &node.kind {
                crate::mes::NodeKind::Command(command) if command.opcode == 0x0a => {
                    command.first_string().and_then(|value| value.text())
                }
                _ => None,
            })
            .next()
            .unwrap();
        assert_eq!(reading, "　　");

        let mut script = MesScript::parse(&source, "cp932").unwrap();
        json[0]["message"] = Value::String("訳文\\n後".to_owned());
        let stats = inject_entries(&mut script, &json.to_string(), "RUBY.MES").unwrap();
        assert_eq!(stats.patched, 1);
        let rebuilt = script.to_bytes().unwrap();
        let reparsed = MesScript::parse(&rebuilt, "cp932").unwrap();
        let result = extract_entries(&reparsed, "RUBY.MES");
        let value = serde_json::to_value(&result.entries).unwrap();
        assert_eq!(value[0]["message"], "訳文\\n後");
    }

    #[test]
    fn control_deletion_and_encoding_failure_are_fatal() {
        let source = ruby_fixture();
        let script = MesScript::parse(&source, "cp932").unwrap();
        let extracted = extract_entries(&script, "RUBY.MES");
        let mut json = serde_json::to_value(&extracted.entries).unwrap();
        json[0]["message"] = Value::String("控制符被删".to_owned());
        let mut target = script.clone();
        assert!(inject_entries(&mut target, &json.to_string(), "RUBY.MES").is_err());

        let mut json = serde_json::to_value(&extracted.entries).unwrap();
        json[0]["message"] = Value::String("简\\n後".to_owned());
        let mut target = script;
        assert!(inject_entries(&mut target, &json.to_string(), "RUBY.MES").is_err());
    }

    #[test]
    fn malformed_ruby_refuses_partial_injection() {
        let source = ruby_fixture();
        let mut script = MesScript::parse(&source, "cp932").unwrap();
        let base_node = script
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| match &node.kind {
                crate::mes::NodeKind::Command(command) if command.opcode == 0x0a => Some(index),
                _ => None,
            })
            .next_back()
            .unwrap();
        script.command_mut(base_node).unwrap().opcode = 0x0b;
        let extracted = extract_entries(&script, "BROKEN.MES");
        assert!(extracted.entries.is_empty());
        assert_eq!(extracted.skipped_blocks, 1);
        assert!(inject_entries(&mut script, "[]", "BROKEN.MES").is_err());
    }
}
