//! JACK resource projections and guarded rebuilding for J no Higeki.

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use vn_font::font_98::EncodingPlan;

pub const ATX_FORMAT: &str = "j-no-higeki-atx-text-v1";
pub const ADW_FORMAT: &str = "j-no-higeki-adw-text-v1";
pub const EXE_FORMAT: &str = "j-no-higeki-exe-ui-text-v1";
const ATX_ENCODING: &str = "bytewise NOT, then strict CP932";
const ATX_STRUCTURE: &str =
    "quoted text in MSG/THING declarations and scene instruction streams; only message is editable";
const ADW_STRUCTURE: &str =
    "one display word followed by whitespace-separated input aliases per non-empty line; only message is editable";
const EXE_STRUCTURE: &str =
    "fixed-width JACK.EXE UI field; translated bytes are space-padded to the original capacity";
const SELECT_MENU: &[u8] = b"Select Menu:  \0";

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtxEntry {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_line")]
    pub line: usize,
    #[serde(rename = "_kind")]
    pub kind: String,
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "_scene", skip_serializing_if = "Option::is_none")]
    pub scene: Option<String>,
    pub scr_msg: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtxJson {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_source_size")]
    pub source_size: usize,
    #[serde(rename = "_source_sha256")]
    pub source_sha256: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_structure")]
    pub structure: String,
    pub entries: Vec<AtxEntry>,
    pub logical_entries: Vec<AtxLogicalEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtxLogicalPart {
    #[serde(rename = "_type")]
    pub kind: String,
    #[serde(rename = "_entry")]
    pub entry: usize,
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtxLogicalEntry {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_line")]
    pub line: usize,
    #[serde(rename = "_scene", skip_serializing_if = "Option::is_none")]
    pub scene: Option<String>,
    #[serde(rename = "_source_offset")]
    pub source_offset: usize,
    #[serde(rename = "_source_size")]
    pub source_size: usize,
    pub scr_msg: String,
    pub message: String,
    pub parts: Vec<AtxLogicalPart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdwEntry {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_line")]
    pub line: usize,
    #[serde(rename = "_aliases")]
    pub aliases: Vec<String>,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdwJson {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_source_size")]
    pub source_size: usize,
    #[serde(rename = "_source_sha256")]
    pub source_sha256: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_structure")]
    pub structure: String,
    pub entries: Vec<AdwEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExeEntry {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_capacity")]
    pub capacity: usize,
    #[serde(rename = "_padding")]
    pub padding: String,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExeJson {
    #[serde(rename = "_format")]
    pub format: String,
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_source_size")]
    pub source_size: usize,
    #[serde(rename = "_source_sha256")]
    pub source_sha256: String,
    #[serde(rename = "_encoding")]
    pub encoding: String,
    #[serde(rename = "_structure")]
    pub structure: String,
    pub entries: Vec<ExeEntry>,
}

#[derive(Debug, Clone)]
pub struct MessageDecl {
    pub name: String,
    pub source_offset: usize,
    pub prefix_length: usize,
}

#[derive(Debug, Clone)]
pub struct SceneDecl {
    pub id: usize,
    pub source_offset: usize,
    pub source_line: usize,
}

#[derive(Debug, Clone)]
pub struct ParsedAtx {
    pub json: AtxJson,
    pub messages: Vec<MessageDecl>,
    pub scenes: Vec<SceneDecl>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenKind {
    Word,
    Number,
    String,
    Symbol,
    StrayQuote,
    DosEof,
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    text: String,
    offset: usize,
    payload_offset: usize,
    payload_size: usize,
    line: usize,
}

#[derive(Debug, Clone)]
struct RawLogicalPart {
    kind: String,
    entry_id: String,
    text: String,
}

#[derive(Debug, Clone)]
struct RawLogicalEntry {
    line: usize,
    scene: Option<String>,
    source_offset: usize,
    source_size: usize,
    parts: Vec<RawLogicalPart>,
}

#[derive(Clone, Copy)]
struct CharPos {
    ch: char,
    offset: usize,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn decrypt_not(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().map(|byte| byte ^ 0xff).collect()
}

fn encrypt_not(bytes: &[u8]) -> Vec<u8> {
    decrypt_not(bytes)
}

fn decode_cp932(bytes: &[u8], context: &str) -> Result<String> {
    let decoded = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .ok_or_else(|| format!("{context} 不是严格 CP932"))?;
    let (encoded, _, errors) = SHIFT_JIS.encode(&decoded);
    if errors || encoded.as_ref() != bytes {
        return Err(format!("{context} 不能按 CP932 原样往返"));
    }
    Ok(decoded.into_owned())
}

fn encode_native_cp932(text: &str, context: &str) -> Result<Vec<u8>> {
    let (encoded, _, errors) = SHIFT_JIS.encode(text);
    if errors {
        return Err(format!("{context} 含 CP932 无法编码的字符"));
    }
    Ok(encoded.into_owned())
}

fn char_positions(text: &str) -> Result<Vec<CharPos>> {
    let mut offset = 0usize;
    let mut chars = Vec::with_capacity(text.chars().count());
    for ch in text.chars() {
        chars.push(CharPos { ch, offset });
        offset += encode_native_cp932(&ch.to_string(), "ATX 字符")?.len();
    }
    Ok(chars)
}

fn is_word(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric() || ch as u32 >= 0x80
}

fn collect_text(chars: &[CharPos], start: usize, end: usize) -> String {
    chars[start..end].iter().map(|item| item.ch).collect()
}

fn token_end(token: &Token) -> usize {
    if token.kind == TokenKind::String {
        token.payload_offset + token.payload_size + 1
    } else {
        token.offset + token.payload_size
    }
}

fn lex_atx(text: &str, plain_len: usize) -> Result<Vec<Token>> {
    let chars = char_positions(text)?;
    let mut tokens = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    while i < chars.len() {
        let ch = chars[i].ch;
        if matches!(ch, ' ' | '\t') {
            i += 1;
            continue;
        }
        if ch == '\r' || ch == '\n' {
            if ch == '\r' && chars.get(i + 1).is_some_and(|item| item.ch == '\n') {
                i += 2;
            } else {
                i += 1;
            }
            line += 1;
            continue;
        }
        if ch == '*' {
            let start_line = line;
            i += 1;
            let mut closed = false;
            while i < chars.len() {
                match chars[i].ch {
                    '*' => {
                        i += 1;
                        closed = true;
                        break;
                    }
                    '\r' => {
                        if chars.get(i + 1).is_some_and(|item| item.ch == '\n') {
                            i += 2;
                        } else {
                            i += 1;
                        }
                        line += 1;
                    }
                    '\n' => {
                        i += 1;
                        line += 1;
                    }
                    _ => i += 1,
                }
            }
            if !closed {
                return Err(format!("ATX 第 {start_line} 行注释没有结束星号"));
            }
            continue;
        }
        let start = i;
        let offset = chars[i].offset;
        if ch == '"' {
            let mut end = i + 1;
            while end < chars.len() && !matches!(chars[end].ch, '"' | '\r' | '\n') {
                end += 1;
            }
            if end >= chars.len() || chars[end].ch != '"' {
                tokens.push(Token {
                    kind: TokenKind::StrayQuote,
                    text: "\"".to_owned(),
                    offset,
                    payload_offset: offset,
                    payload_size: 0,
                    line,
                });
                i += 1;
                continue;
            }
            let payload_offset = chars.get(i + 1).map_or(offset + 1, |item| item.offset);
            let closing_offset = chars[end].offset;
            tokens.push(Token {
                kind: TokenKind::String,
                text: collect_text(&chars, i + 1, end),
                offset,
                payload_offset,
                payload_size: closing_offset - payload_offset,
                line,
            });
            i = end + 1;
            continue;
        }
        let (kind, end) = if ch.is_ascii_digit() {
            let mut end = i + 1;
            while end < chars.len() && chars[end].ch.is_ascii_digit() {
                end += 1;
            }
            (TokenKind::Number, end)
        } else if is_word(ch) {
            let mut end = i + 1;
            while end < chars.len() && is_word(chars[end].ch) {
                end += 1;
            }
            (TokenKind::Word, end)
        } else if ch == '\u{1a}' {
            (TokenKind::DosEof, i + 1)
        } else {
            (TokenKind::Symbol, i + 1)
        };
        let end_offset = chars.get(end).map_or(plain_len, |item| item.offset);
        tokens.push(Token {
            kind,
            text: collect_text(&chars, start, end),
            offset,
            payload_offset: offset,
            payload_size: end_offset - offset,
            line,
        });
        i = end;
    }
    Ok(tokens)
}

fn add_atx_entry(
    entries: &mut Vec<AtxEntry>,
    token: &Token,
    kind: &str,
    id: Option<String>,
    scene: Option<String>,
) {
    entries.push(AtxEntry {
        index: entries.len(),
        offset: token.payload_offset,
        size: token.payload_size,
        line: token.line,
        kind: kind.to_owned(),
        id,
        scene,
        scr_msg: token.text.clone(),
        message: token.text.clone(),
    });
}

pub fn parse_atx(bytes: &[u8], file: String) -> Result<ParsedAtx> {
    let plain = decrypt_not(bytes);
    let text = decode_cp932(&plain, &file)?;
    let tokens = lex_atx(&text, plain.len())?;
    let mut by_line = BTreeMap::<usize, Vec<&Token>>::new();
    for token in &tokens {
        by_line.entry(token.line).or_default().push(token);
    }
    let mut entries = Vec::new();
    let mut messages: Vec<MessageDecl> = Vec::new();
    let mut scenes: Vec<SceneDecl> = Vec::new();
    let mut pending_scene = None::<String>;
    let mut current_scene = None::<String>;
    let mut in_scene = false;
    let mut raw_logical_entries = Vec::<RawLogicalEntry>::new();

    for line_tokens in by_line.values() {
        let first = line_tokens[0];
        if first.text == "["
            && line_tokens.len() >= 3
            && line_tokens[1].kind == TokenKind::Number
            && line_tokens[2].text == "]"
        {
            let id_text = line_tokens[1].text.clone();
            let id = id_text
                .parse::<usize>()
                .map_err(|_| format!("ATX 第 {} 行场景号无效", first.line))?;
            scenes.push(SceneDecl {
                id,
                source_offset: first.offset,
                source_line: first.line,
            });
            pending_scene = Some(id_text);
            continue;
        }
        if line_tokens.len() == 1 && first.text == "{" {
            current_scene = pending_scene.clone();
            in_scene = true;
            continue;
        }
        if line_tokens.len() == 1 && first.text == "}" {
            current_scene = None;
            pending_scene = None;
            in_scene = false;
            continue;
        }
        if in_scene {
            let output_start = line_tokens
                .iter()
                .find(|token| token.text == "?")
                .map(|token| token.offset);
            let mut raw_parts = Vec::new();
            for token in line_tokens
                .iter()
                .copied()
                .filter(|token| token.kind == TokenKind::String)
            {
                add_atx_entry(
                    &mut entries,
                    token,
                    "inline_text",
                    None,
                    current_scene.clone(),
                );
            }
            if let Some(output_start) = output_start {
                let message_names = messages
                    .iter()
                    .map(|message| message.name.as_str())
                    .collect::<BTreeSet<_>>();
                let mut started = false;
                for token in line_tokens.iter().copied() {
                    if token.text == "?" {
                        started = true;
                        continue;
                    }
                    if !started {
                        continue;
                    }
                    if token.kind == TokenKind::String {
                        if let Some(entry) = entries.iter().find(|entry| {
                            entry.offset == token.payload_offset && entry.kind == "inline_text"
                        }) {
                            raw_parts.push(RawLogicalPart {
                                kind: "text".to_owned(),
                                entry_id: entry.offset.to_string(),
                                text: token.text.clone(),
                            });
                        }
                    } else if token.kind == TokenKind::Word
                        && message_names.contains(token.text.as_str())
                    {
                        if let Some(entry) = entries.iter().find(|entry| {
                            entry.kind == "message"
                                && entry.id.as_deref() == Some(token.text.as_str())
                        }) {
                            raw_parts.push(RawLogicalPart {
                                kind: "msg".to_owned(),
                                entry_id: entry.offset.to_string(),
                                text: entry.scr_msg.clone(),
                            });
                        }
                    }
                }
                if raw_parts.len() >= 2 {
                    let source_end = line_tokens
                        .iter()
                        .rev()
                        .find(|token| {
                            token.kind == TokenKind::String
                                || (token.kind == TokenKind::Word
                                    && message_names.contains(token.text.as_str()))
                        })
                        .map(|token| token_end(token))
                        .unwrap_or(output_start);
                    raw_logical_entries.push(RawLogicalEntry {
                        line: first.line,
                        scene: current_scene.clone(),
                        source_offset: output_start,
                        source_size: source_end.saturating_sub(output_start),
                        parts: raw_parts,
                    });
                }
            }
            continue;
        }
        if first.kind == TokenKind::Word && first.text == "MSG" {
            if line_tokens.len() < 3
                || line_tokens[1].kind != TokenKind::Word
                || line_tokens[2].kind != TokenKind::String
            {
                return Err(format!("ATX 第 {} 行 MSG 声明无效", first.line));
            }
            let name = line_tokens[1].text.clone();
            let token = line_tokens[2];
            messages.push(MessageDecl {
                name: name.clone(),
                source_offset: first.offset,
                prefix_length: encode_native_cp932(&format!("MSG {name}"), "MSG 前缀")?.len(),
            });
            add_atx_entry(&mut entries, token, "message", Some(name), None);
            continue;
        }
        if first.kind == TokenKind::Word && first.text == "THING" {
            let mut i = 1usize;
            while i < line_tokens.len() {
                if line_tokens[i].kind != TokenKind::Word {
                    i += 1;
                    continue;
                }
                if i + 4 >= line_tokens.len()
                    || line_tokens[i + 1].text != "="
                    || line_tokens[i + 3].text != ":"
                    || line_tokens[i + 4].kind != TokenKind::String
                {
                    return Err(format!("ATX 第 {} 行 THING 声明无效", first.line));
                }
                add_atx_entry(
                    &mut entries,
                    line_tokens[i + 4],
                    "thing_name",
                    Some(line_tokens[i].text.clone()),
                    None,
                );
                i += 5;
            }
        }
    }
    entries.sort_by_key(|entry| entry.offset);
    for (index, entry) in entries.iter_mut().enumerate() {
        entry.index = index;
    }
    let mut logical_entries = Vec::new();
    for raw in raw_logical_entries {
        let parts = raw
            .parts
            .into_iter()
            .map(|part| {
                let offset = part
                    .entry_id
                    .parse::<usize>()
                    .map_err(|_| "ATX 逻辑分段偏移无效".to_owned())?;
                let entry = entries
                    .iter()
                    .find(|entry| entry.offset == offset)
                    .ok_or_else(|| "ATX 逻辑分段引用越界".to_owned())?;
                Ok(AtxLogicalPart {
                    kind: part.kind,
                    entry: entry.index,
                    id: entry.id.clone(),
                    text: part.text,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let scr_msg = parts
            .iter()
            .map(|part| part.text.as_str())
            .collect::<String>();
        logical_entries.push(AtxLogicalEntry {
            index: logical_entries.len(),
            line: raw.line,
            scene: raw.scene,
            source_offset: raw.source_offset,
            source_size: raw.source_size,
            scr_msg: scr_msg.clone(),
            message: scr_msg,
            parts,
        });
    }
    // The grouped entry is the editable, user-facing sentence. Keep the
    // original atom metadata for validation and rebuilding, but do not emit a
    // second editable `message` for every atom inside the sentence.
    let grouped_entries = logical_entries
        .iter()
        .flat_map(|logical| logical.parts.iter().map(|part| part.entry))
        .collect::<BTreeSet<_>>();
    for entry in &mut entries {
        if grouped_entries.contains(&entry.index) {
            entry.message.clear();
        }
    }
    Ok(ParsedAtx {
        json: AtxJson {
            format: ATX_FORMAT.to_owned(),
            file,
            source_size: bytes.len(),
            source_sha256: sha256(bytes),
            encoding: ATX_ENCODING.to_owned(),
            structure: ATX_STRUCTURE.to_owned(),
            entries,
            logical_entries,
        },
        messages,
        scenes,
    })
}

pub fn parse_adw(bytes: &[u8], file: String) -> Result<AdwJson> {
    let plain = decrypt_not(bytes);
    decode_cp932(&plain, &file)?;
    let mut entries = Vec::new();
    let mut line = 1usize;
    let mut start = 0usize;
    while start < plain.len() {
        let mut end = start;
        while end < plain.len() && !matches!(plain[end], b'\r' | b'\n') {
            end += 1;
        }
        let mut logical_end = end;
        while logical_end > start && plain[logical_end - 1] == 0x1a {
            logical_end -= 1;
        }
        let mut word_start = start;
        while word_start < logical_end && matches!(plain[word_start], b' ' | b'\t') {
            word_start += 1;
        }
        if word_start < logical_end {
            let mut word_end = word_start;
            while word_end < logical_end && !matches!(plain[word_end], b' ' | b'\t' | 0x1a) {
                word_end += 1;
            }
            let display = decode_cp932(&plain[word_start..word_end], "ADW 显示词")?;
            let rest = decode_cp932(&plain[word_end..logical_end], "ADW 别名")?;
            let aliases = rest
                .split_ascii_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            entries.push(AdwEntry {
                index: entries.len(),
                offset: word_start,
                size: word_end - word_start,
                line,
                aliases,
                scr_msg: display.clone(),
                message: display,
            });
        }
        if end >= plain.len() {
            break;
        }
        if plain[end] == b'\r' && plain.get(end + 1) == Some(&b'\n') {
            start = end + 2;
        } else {
            start = end + 1;
        }
        line += 1;
    }
    Ok(AdwJson {
        format: ADW_FORMAT.to_owned(),
        file,
        source_size: bytes.len(),
        source_sha256: sha256(bytes),
        encoding: ATX_ENCODING.to_owned(),
        structure: ADW_STRUCTURE.to_owned(),
        entries,
    })
}

pub fn parse_exe(bytes: &[u8], file: String) -> Result<ExeJson> {
    let offsets = bytes
        .windows(SELECT_MENU.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == SELECT_MENU).then_some(offset))
        .collect::<Vec<_>>();
    if offsets.len() != 1 {
        return Err(format!(
            "{file}: 固定 UI 文本 Select Menu 唯一匹配数应为 1，实际 {}",
            offsets.len()
        ));
    }
    Ok(ExeJson {
        format: EXE_FORMAT.to_owned(),
        file,
        source_size: bytes.len(),
        source_sha256: sha256(bytes),
        encoding: "fixed-width CP932 carriers".to_owned(),
        structure: EXE_STRUCTURE.to_owned(),
        entries: vec![ExeEntry {
            index: 0,
            offset: offsets[0],
            capacity: SELECT_MENU.len() - 1,
            padding: "ASCII space".to_owned(),
            scr_msg: "Select Menu:".to_owned(),
            message: "Select Menu:".to_owned(),
        }],
    })
}

fn same_atx_metadata(actual: &AtxJson, expected: &AtxJson) -> bool {
    actual.format == expected.format
        && actual.file == expected.file
        && actual.source_size == expected.source_size
        && actual
            .source_sha256
            .eq_ignore_ascii_case(&expected.source_sha256)
        && actual.encoding == expected.encoding
        && actual.structure == expected.structure
        && actual.entries.len() == expected.entries.len()
        && actual
            .entries
            .iter()
            .zip(&expected.entries)
            .all(|(left, right)| {
                left.index == right.index
                    && left.offset == right.offset
                    && left.size == right.size
                    && left.line == right.line
                    && left.kind == right.kind
                    && left.id == right.id
                    && left.scene == right.scene
                    && left.scr_msg == right.scr_msg
            })
        && actual.logical_entries.len() == expected.logical_entries.len()
        && actual
            .logical_entries
            .iter()
            .zip(&expected.logical_entries)
            .all(|(left, right)| {
                left.index == right.index
                    && left.line == right.line
                    && left.scene == right.scene
                    && left.source_offset == right.source_offset
                    && left.source_size == right.source_size
                    && left.scr_msg == right.scr_msg
                    && left.parts.len() == right.parts.len()
                    && left.parts.iter().zip(&right.parts).all(|(a, b)| {
                        a.kind == b.kind && a.entry == b.entry && a.id == b.id && a.text == b.text
                    })
            })
}

pub fn validate_atx(script: &AtxJson, source: &[u8]) -> Result<ParsedAtx> {
    let expected = parse_atx(source, script.file.clone())?;
    if !same_atx_metadata(script, &expected.json) {
        return Err("ATX JSON 的源元数据、条目结构或 scr_msg 已被修改".to_owned());
    }
    for entry in &script.entries {
        if entry.message.is_empty() {
            continue;
        }
        normalize_translation(&entry.message)
            .map_err(|error| format!("ATX entry {} (line {}): {error}", entry.index, entry.line))?;
    }
    for logical in &script.logical_entries {
        normalize_translation(&logical.message).map_err(|error| {
            format!(
                "ATX logical entry {} (line {}): {error}",
                logical.index, logical.line
            )
        })?;
    }
    Ok(expected)
}

fn same_adw_metadata(actual: &AdwJson, expected: &AdwJson) -> bool {
    actual.format == expected.format
        && actual.file == expected.file
        && actual.source_size == expected.source_size
        && actual
            .source_sha256
            .eq_ignore_ascii_case(&expected.source_sha256)
        && actual.encoding == expected.encoding
        && actual.structure == expected.structure
        && actual.entries.len() == expected.entries.len()
        && actual
            .entries
            .iter()
            .zip(&expected.entries)
            .all(|(left, right)| {
                left.index == right.index
                    && left.offset == right.offset
                    && left.size == right.size
                    && left.line == right.line
                    && left.aliases == right.aliases
                    && left.scr_msg == right.scr_msg
            })
}

pub fn validate_adw(script: &AdwJson, source: &[u8]) -> Result<AdwJson> {
    let expected = parse_adw(source, script.file.clone())?;
    if !same_adw_metadata(script, &expected) {
        return Err("ADW JSON 的源元数据、条目结构、别名或 scr_msg 已被修改".to_owned());
    }
    for entry in &script.entries {
        if entry.message == entry.scr_msg {
            continue;
        }
        let normalized = normalize_translation(&entry.message)
            .map_err(|error| format!("ADW entry {}: {error}", entry.index))?;
        if normalized
            .chars()
            .any(|character| character.is_ascii_whitespace())
        {
            return Err(format!("ADW entry {} 不能包含半角空白", entry.index));
        }
    }
    Ok(expected)
}

pub fn validate_exe(script: &ExeJson, source: &[u8]) -> Result<ExeJson> {
    let expected = parse_exe(source, script.file.clone())?;
    if script.format != expected.format
        || script.file != expected.file
        || script.source_size != expected.source_size
        || !script
            .source_sha256
            .eq_ignore_ascii_case(&expected.source_sha256)
        || script.encoding != expected.encoding
        || script.structure != expected.structure
        || script.entries.len() != 1
        || script.entries[0].index != expected.entries[0].index
        || script.entries[0].offset != expected.entries[0].offset
        || script.entries[0].capacity != expected.entries[0].capacity
        || script.entries[0].padding != expected.entries[0].padding
        || script.entries[0].scr_msg != expected.entries[0].scr_msg
    {
        return Err("EXE UI JSON 的源元数据、固定容量或 scr_msg 已被修改".to_owned());
    }
    normalize_translation(&script.entries[0].message)?;
    Ok(expected)
}

pub fn normalize_translation(value: &str) -> Result<String> {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        let normalized = match character {
            '\0' | '\r' | '\n' | '"' | '\u{1a}' => {
                return Err("message 不能包含 NUL、换行、双引号或 DOS EOF".to_owned())
            }
            ' ' => '　',
            '!'..='~' => char::from_u32(character as u32 + 0xfee0).expect("ASCII 全角映射始终有效"),
            value if value.is_control() => {
                return Err(format!("message 含控制字符 U+{:04X}", value as u32))
            }
            value => value,
        };
        output.push(normalized);
    }
    Ok(output)
}

fn rebuild_ranges<T>(
    plain: &[u8],
    entries: &[T],
    range: impl Fn(&T) -> (usize, usize),
    replacement: impl Fn(&T) -> Result<Option<Vec<u8>>>,
) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(plain.len());
    let mut cursor = 0usize;
    for entry in entries {
        let (offset, size) = range(entry);
        if offset < cursor || offset + size > plain.len() {
            return Err("文本条目范围重叠或越界".to_owned());
        }
        output.extend_from_slice(&plain[cursor..offset]);
        if let Some(bytes) = replacement(entry)? {
            output.extend_from_slice(&bytes);
        } else {
            output.extend_from_slice(&plain[offset..offset + size]);
        }
        cursor = offset + size;
    }
    output.extend_from_slice(&plain[cursor..]);
    Ok(output)
}

pub fn rebuild_atx(script: &AtxJson, source: &[u8], plan: &EncodingPlan) -> Result<Vec<u8>> {
    validate_atx(script, source)?;
    let plain = decrypt_not(source);
    let changed_groups = script
        .logical_entries
        .iter()
        .filter(|logical| logical.message != logical.scr_msg)
        .collect::<Vec<_>>();
    let mut covered_literal_entries = BTreeSet::new();
    let mut patches = Vec::<(usize, usize, Vec<u8>)>::new();
    for logical in changed_groups {
        for part in &logical.parts {
            if part.kind == "text" {
                covered_literal_entries.insert(part.entry);
            }
        }
        let normalized = normalize_translation(&logical.message)?;
        let mut replacement = vec![b'?', b'"'];
        replacement.extend_from_slice(&plan.encode_cp932(&normalized)?);
        replacement.push(b'"');
        patches.push((logical.source_offset, logical.source_size, replacement));
    }
    for entry in &script.entries {
        if covered_literal_entries.contains(&entry.index)
            || entry.message.is_empty()
            || entry.message == entry.scr_msg
        {
            continue;
        }
        let normalized = normalize_translation(&entry.message)?;
        patches.push((entry.offset, entry.size, plan.encode_cp932(&normalized)?));
    }
    patches.sort_by_key(|patch| patch.0);
    let mut rebuilt = Vec::with_capacity(plain.len());
    let mut cursor = 0usize;
    for (offset, size, replacement) in patches {
        if offset < cursor || offset + size > plain.len() {
            return Err("ATX 回写范围重叠或越界".to_owned());
        }
        rebuilt.extend_from_slice(&plain[cursor..offset]);
        rebuilt.extend_from_slice(&replacement);
        cursor = offset + size;
    }
    rebuilt.extend_from_slice(&plain[cursor..]);
    Ok(encrypt_not(&rebuilt))
}

pub fn rebuild_adw(script: &AdwJson, source: &[u8], plan: &EncodingPlan) -> Result<Vec<u8>> {
    validate_adw(script, source)?;
    let plain = decrypt_not(source);
    let rebuilt = rebuild_ranges(
        &plain,
        &script.entries,
        |entry| (entry.offset, entry.size),
        |entry| {
            if entry.message == entry.scr_msg {
                Ok(None)
            } else {
                let normalized = normalize_translation(&entry.message)?;
                if normalized
                    .chars()
                    .any(|character| character.is_ascii_whitespace())
                {
                    return Err(format!("ADW entry {} 不能包含半角空白", entry.index));
                }
                Ok(Some(plan.encode_cp932(&normalized)?))
            }
        },
    )?;
    Ok(encrypt_not(&rebuilt))
}

pub fn rebuild_exe(script: &ExeJson, source: &[u8], plan: &EncodingPlan) -> Result<Vec<u8>> {
    validate_exe(script, source)?;
    let entry = &script.entries[0];
    if entry.message == entry.scr_msg {
        return Ok(source.to_vec());
    }
    let normalized = normalize_translation(&entry.message)?;
    let encoded = plan.encode_cp932(&normalized)?;
    if encoded.len() > entry.capacity {
        return Err(format!(
            "EXE UI 文本编码后 {} 字节，超过固定容量 {} 字节",
            encoded.len(),
            entry.capacity
        ));
    }
    let mut output = source.to_vec();
    output[entry.offset..entry.offset + entry.capacity].fill(b' ');
    output[entry.offset..entry.offset + encoded.len()].copy_from_slice(&encoded);
    Ok(output)
}

pub fn changed_atx(script: &AtxJson) -> usize {
    let grouped = script
        .logical_entries
        .iter()
        .filter(|logical| logical.message != logical.scr_msg)
        .flat_map(|logical| {
            logical
                .parts
                .iter()
                .filter(|part| part.kind == "text")
                .map(|part| part.entry)
        })
        .collect::<BTreeSet<_>>();
    script
        .logical_entries
        .iter()
        .filter(|logical| logical.message != logical.scr_msg)
        .count()
        + script
            .entries
            .iter()
            .filter(|entry| {
                !entry.message.is_empty()
                    && entry.message != entry.scr_msg
                    && !grouped.contains(&entry.index)
            })
            .count()
}

pub fn changed_adw(script: &AdwJson) -> usize {
    script
        .entries
        .iter()
        .filter(|entry| entry.message != entry.scr_msg)
        .count()
}

pub fn changed_exe(script: &ExeJson) -> usize {
    usize::from(script.entries[0].message != script.entries[0].scr_msg)
}

pub fn final_changed_texts(atx: &AtxJson, adw: &AdwJson, exe: &ExeJson) -> Result<Vec<String>> {
    let mut texts = Vec::new();
    let grouped = atx
        .logical_entries
        .iter()
        .filter(|logical| logical.message != logical.scr_msg)
        .flat_map(|logical| {
            logical
                .parts
                .iter()
                .filter(|part| part.kind == "text")
                .map(|part| part.entry)
        })
        .collect::<BTreeSet<_>>();
    for logical in &atx.logical_entries {
        if logical.message != logical.scr_msg {
            texts.push(normalize_translation(&logical.message)?);
        }
    }
    for (index, (message, scr_msg)) in atx
        .entries
        .iter()
        .enumerate()
        .map(|(index, entry)| (index, (&entry.message, &entry.scr_msg)))
        .chain(
            adw.entries
                .iter()
                .map(|entry| (usize::MAX, (&entry.message, &entry.scr_msg))),
        )
        .chain(
            exe.entries
                .iter()
                .map(|entry| (usize::MAX, (&entry.message, &entry.scr_msg))),
        )
    {
        if !message.is_empty()
            && message != scr_msg
            && (index == usize::MAX || !grouped.contains(&index))
        {
            texts.push(normalize_translation(message)?);
        }
    }
    Ok(texts)
}

fn reserve_text(text: &str, reserved: &mut BTreeSet<u16>) -> Result<()> {
    for character in text.chars() {
        let encoded = encode_native_cp932(&character.to_string(), "源文字")?;
        if encoded.len() == 2 {
            reserved.insert(u16::from_be_bytes([encoded[0], encoded[1]]));
        }
    }
    Ok(())
}

pub fn source_reserved_cp932(atx: &AtxJson, adw: &AdwJson, exe: &ExeJson) -> Result<BTreeSet<u16>> {
    let mut reserved = BTreeSet::new();
    for entry in &atx.entries {
        if !entry.message.is_empty() && entry.message == entry.scr_msg {
            reserve_text(&entry.scr_msg, &mut reserved)?;
        }
    }
    for logical in &atx.logical_entries {
        if logical.message == logical.scr_msg {
            reserve_text(&logical.scr_msg, &mut reserved)?;
        }
    }
    for entry in &adw.entries {
        if entry.message == entry.scr_msg {
            reserve_text(&entry.scr_msg, &mut reserved)?;
        }
        for alias in &entry.aliases {
            reserve_text(alias, &mut reserved)?;
        }
    }
    for entry in &exe.entries {
        if entry.message == entry.scr_msg {
            reserve_text(&entry.scr_msg, &mut reserved)?;
        }
    }
    Ok(reserved)
}

fn split_pointer(value: usize, context: &str) -> Result<(u16, u16)> {
    let paragraph = value / 16;
    let offset = value % 16;
    Ok((
        u16::try_from(offset).expect("paragraph offset is below 16"),
        u16::try_from(paragraph)
            .map_err(|_| format!("{context} 的 ATX 偏移 0x{value:X} 超过索引容量"))?,
    ))
}

pub fn validate_msg(source: &[u8], atx: &ParsedAtx) -> Result<()> {
    let plain = decrypt_not(source);
    if plain.len() != atx.messages.len() * 16 {
        return Err(format!(
            "JACK_C.MSG 应有 {} 个 16 字节记录，实际 {} 字节",
            atx.messages.len(),
            plain.len()
        ));
    }
    for (index, expected) in atx.messages.iter().enumerate() {
        let record = &plain[index * 16..index * 16 + 16];
        let name_end = record[..10]
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(10);
        let name = std::str::from_utf8(&record[..name_end])
            .map_err(|_| format!("MSG record {index} 名称不是 ASCII"))?;
        let source_offset = usize::from(u16::from_le_bytes([record[10], record[11]]))
            + usize::from(u16::from_le_bytes([record[12], record[13]])) * 16;
        let prefix_length = usize::from(u16::from_le_bytes([record[14], record[15]]));
        if name != expected.name
            || source_offset != expected.source_offset
            || prefix_length != expected.prefix_length
        {
            return Err(format!("MSG record {index} 与 ATX 声明不匹配"));
        }
    }
    Ok(())
}

pub fn rebuild_msg(source: &[u8], atx: &ParsedAtx) -> Result<Vec<u8>> {
    if source.len() != atx.messages.len() * 16 {
        return Err("JACK_C.MSG 记录数与 ATX MSG 声明数不匹配".to_owned());
    }
    // The shipped name fields contain harmless bytes after their first NUL.
    // Names and prefix lengths are immutable, so retain those bytes exactly and
    // update only the two paragraph-pointer words affected by variable text.
    let mut plain = decrypt_not(source);
    for (index, message) in atx.messages.iter().enumerate() {
        let record = &mut plain[index * 16..index * 16 + 16];
        let (offset, paragraph) = split_pointer(message.source_offset, "MSG")?;
        record[10..12].copy_from_slice(&offset.to_le_bytes());
        record[12..14].copy_from_slice(&paragraph.to_le_bytes());
    }
    Ok(encrypt_not(&plain))
}

pub fn validate_frm(source: &[u8], atx: &ParsedAtx) -> Result<()> {
    let plain = decrypt_not(source);
    if !plain.len().is_multiple_of(8) {
        return Err("JACK_C.FRM 长度不是 8 的倍数".to_owned());
    }
    for scene in &atx.scenes {
        let record = plain
            .get(scene.id * 8..scene.id * 8 + 8)
            .ok_or_else(|| format!("FRM 缺少场景 {} 的索引槽", scene.id))?;
        let source_offset = usize::from(u16::from_le_bytes([record[0], record[1]]))
            + usize::from(u16::from_le_bytes([record[2], record[3]])) * 16;
        let source_line = usize::from(u16::from_le_bytes([record[4], record[5]]));
        if source_offset != scene.source_offset || source_line != scene.source_line {
            return Err(format!("FRM 场景 {} 与 ATX 声明不匹配", scene.id));
        }
    }
    Ok(())
}

pub fn rebuild_frm(source: &[u8], atx: &ParsedAtx) -> Result<Vec<u8>> {
    if !source.len().is_multiple_of(8) {
        return Err("JACK_C.FRM 长度不是 8 的倍数".to_owned());
    }
    let mut plain = decrypt_not(source);
    for scene in &atx.scenes {
        let record = plain
            .get_mut(scene.id * 8..scene.id * 8 + 8)
            .ok_or_else(|| format!("FRM 缺少场景 {} 的索引槽", scene.id))?;
        let (offset, paragraph) = split_pointer(scene.source_offset, "FRM")?;
        record[0..2].copy_from_slice(&offset.to_le_bytes());
        record[2..4].copy_from_slice(&paragraph.to_le_bytes());
        let line = u16::try_from(scene.source_line)
            .map_err(|_| format!("FRM 场景 {} 行号超限", scene.id))?;
        record[4..6].copy_from_slice(&line.to_le_bytes());
    }
    Ok(encrypt_not(&plain))
}

pub fn validate_flg(source: &[u8]) -> Result<()> {
    let plain = decrypt_not(source);
    if plain.len() < 2 || plain[..2] != [0x01, 0x03] || !(plain.len() - 2).is_multiple_of(16) {
        return Err("JACK_C.FLG 不是 2 字节头加 16 字节定长记录".to_owned());
    }
    for (index, record) in plain[2..].chunks_exact(16).enumerate() {
        if record[14..] != *b": " {
            return Err(format!("FLG record {index} 分隔符无效"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vn_font::font_98::SubstitutionMap;

    fn not(text: &str) -> Vec<u8> {
        encrypt_not(&encode_native_cp932(text, "test").unwrap())
    }

    #[test]
    fn parses_and_rebuilds_atx_without_touching_anomalies() {
        let source = not(
            "*comment*\r\nTHING TH0=1:\"鍵\"\r\nMSG M0\"原文\"\r\n[000]\r\n{\r\n(X)?\"本文\";\";\r\n}\r\n\x1a",
        );
        let parsed = parse_atx(&source, "JACK_C.ATX".into()).unwrap();
        assert_eq!(parsed.json.entries.len(), 3);
        assert_eq!(parsed.messages.len(), 1);
        assert_eq!(parsed.scenes.len(), 1);
        let plan = EncodingPlan::build(&SubstitutionMap::embedded().unwrap(), [], []).unwrap();
        assert_eq!(rebuild_atx(&parsed.json, &source, &plan).unwrap(), source);
    }

    #[test]
    fn modified_atx_rebuilds_msg_and_frm_offsets() {
        let source = not("MSG M0\"原\";\r\n[000]\r\n{\r\n(X)?\"文\";\r\n}\r\n\x1a");
        let original = parse_atx(&source, "JACK_C.ATX".into()).unwrap();
        let mut edited = original.json.clone();
        edited.entries[0].message = "测试增长".into();
        let texts = final_changed_texts(
            &edited,
            &AdwJson {
                format: ADW_FORMAT.into(),
                file: "x".into(),
                source_size: 0,
                source_sha256: sha256(&[]),
                encoding: ATX_ENCODING.into(),
                structure: ADW_STRUCTURE.into(),
                entries: vec![],
            },
            &ExeJson {
                format: EXE_FORMAT.into(),
                file: "x".into(),
                source_size: 0,
                source_sha256: sha256(&[]),
                encoding: "fixed-width CP932 carriers".into(),
                structure: EXE_STRUCTURE.into(),
                entries: vec![],
            },
        )
        .unwrap();
        let plan = EncodingPlan::build(
            &SubstitutionMap::embedded().unwrap(),
            source_reserved_cp932(
                &edited,
                &AdwJson {
                    format: ADW_FORMAT.into(),
                    file: "x".into(),
                    source_size: 0,
                    source_sha256: sha256(&[]),
                    encoding: ATX_ENCODING.into(),
                    structure: ADW_STRUCTURE.into(),
                    entries: vec![],
                },
                &ExeJson {
                    format: EXE_FORMAT.into(),
                    file: "x".into(),
                    source_size: 0,
                    source_sha256: sha256(&[]),
                    encoding: "fixed-width CP932 carriers".into(),
                    structure: EXE_STRUCTURE.into(),
                    entries: vec![],
                },
            )
            .unwrap(),
            texts.iter().map(String::as_str),
        )
        .unwrap();
        let rebuilt = rebuild_atx(&edited, &source, &plan).unwrap();
        let reparsed = parse_atx(&rebuilt, "JACK_C.ATX".into()).unwrap();
        assert!(reparsed.scenes[0].source_offset > original.scenes[0].source_offset);
        let mut msg_plain = [b' '; 16];
        msg_plain[..3].copy_from_slice(b"M0\0");
        msg_plain[14..16].copy_from_slice(&6u16.to_le_bytes());
        let msg = rebuild_msg(&encrypt_not(&msg_plain), &reparsed).unwrap();
        validate_msg(&msg, &reparsed).unwrap();
        let frm = rebuild_frm(&[0xff; 8], &reparsed).unwrap();
        validate_frm(&frm, &reparsed).unwrap();
    }

    #[test]
    fn adw_preserves_aliases() {
        let source = not("見る みる 見る\r\n取る とる\r\n\x1a");
        let parsed = parse_adw(&source, "JACK_C.ADW".into()).unwrap();
        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.entries[0].aliases, ["みる", "見る"]);
    }

    #[test]
    fn exe_field_is_fixed_width() {
        let mut source = vec![0u8; 100];
        source[20..20 + SELECT_MENU.len()].copy_from_slice(SELECT_MENU);
        let parsed = parse_exe(&source, "JACK.EXE".into()).unwrap();
        assert_eq!(parsed.entries[0].capacity, 14);
    }
}
