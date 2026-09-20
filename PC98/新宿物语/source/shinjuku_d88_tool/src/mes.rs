//! 《新宿物語》MES bytecode text projection and guarded rebuilding.
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use vn_font::font_98::EncodingPlan;

pub const FORMAT: &str = "shinjuku-monogatari-mes-ir-v1";
const ENCODING: &str = "CP932 + custom hiragana 2D..7F; translated text uses double-byte carriers";
const LEXER: &str = "ADV98 sub_D77C: 00..0F=2, 10..1F=2, 21=NUL, 22=handler quote, A5=linebreak";
const HANDLERS: &str = "ADV98V A6..D8 operand ownership profile, 2026-09-13";
const NAME_POLICY: &str =
    "leading fullwidth square brackets ［name］ are structural; name/message writable";
const SPECIAL_DBCS_CONTROLS: [u16; 4] = [0x8197, 0x8190, 0x816f, 0x8170];

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LexToken {
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_role")]
    pub role: String,
    #[serde(rename = "_raw")]
    pub raw: String,
    #[serde(
        rename = "_owner_opcode_index",
        skip_serializing_if = "Option::is_none"
    )]
    pub owner_opcode_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEntry {
    #[serde(rename = "_file")]
    pub file: String,
    #[serde(rename = "_index")]
    pub index: usize,
    #[serde(rename = "_token_start")]
    pub token_start: usize,
    #[serde(rename = "_token_end")]
    pub token_end: usize,
    #[serde(rename = "_offset")]
    pub offset: usize,
    #[serde(rename = "_size")]
    pub size: usize,
    #[serde(rename = "_type")]
    pub entry_type: String,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    pub scr_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptJson {
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
    #[serde(rename = "_lexer")]
    pub lexer: String,
    #[serde(rename = "_handler_profile")]
    pub handler_profile: String,
    #[serde(rename = "_name_policy")]
    pub name_policy: String,
    pub entries: Vec<TextEntry>,
    #[serde(rename = "_tokens")]
    pub tokens: Vec<LexToken>,
    #[serde(rename = "_warnings")]
    pub warnings: Vec<String>,
}

fn is_sjis_lead(byte: u8) -> bool {
    (0x81..=0x9f).contains(&byte) || (0xe0..=0xfc).contains(&byte)
}

fn is_sjis_trail(byte: u8) -> bool {
    (0x40..=0x7e).contains(&byte) || (0x80..=0xfc).contains(&byte)
}

fn strict_sjis_pair(bytes: &[u8], offset: usize) -> bool {
    if offset + 1 >= bytes.len()
        || !is_sjis_lead(bytes[offset])
        || !is_sjis_trail(bytes[offset + 1])
    {
        return false;
    }
    SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&bytes[offset..offset + 2])
        .is_some()
}

fn token_length(bytes: &[u8], offset: usize, warnings: &mut Vec<String>) -> usize {
    let remaining = bytes.len() - offset;
    let byte = bytes[offset];
    let required = match byte {
        0x21 => {
            if let Some(end) = bytes[offset + 1..].iter().position(|&value| value == 0) {
                return end + 2;
            }
            warnings.push(format!("unterminated 21 NUL string at 0x{offset:X}"));
            return remaining;
        }
        0x00..=0x1f => 2,
        0x20 | 0x22..=0x27 => 1,
        0x28 => 2,
        0x29..=0x2c => 3,
        value if is_sjis_lead(value) => 2,
        _ => 1,
    };
    if remaining < required {
        warnings.push(format!(
            "truncated token {byte:02X} at 0x{offset:X}: expected {required}, have {remaining}"
        ));
    }
    required.min(remaining)
}

fn token_role(bytes: &[u8], offset: usize, size: usize) -> &'static str {
    match bytes[offset] {
        0x21 => "operand_string",
        0x00..=0x20 | 0x80 | 0xd9..=0xdf | 0xfd..=0xff => "control",
        0x22 => "quote_marker",
        0x23..=0x2c => "expression",
        0x2d..=0x7f => "text",
        0xa0..=0xa4 => "block_control",
        0xa5 => "line_break",
        0xa6..=0xd8 => "opcode",
        value if is_sjis_lead(value) => {
            if size == 2
                && matches!(
                    &bytes[offset..offset + 2],
                    [0x81, 0x97] | [0x81, 0x90] | [0x81, 0x6f] | [0x81, 0x70]
                )
            {
                "special_control"
            } else if size == 2 && strict_sjis_pair(bytes, offset) {
                "text"
            } else {
                "invalid_sjis"
            }
        }
        _ => "opaque",
    }
}

fn lex_full_file(bytes: &[u8]) -> (Vec<LexToken>, Vec<String>) {
    let mut tokens = Vec::new();
    let mut warnings = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let size = token_length(bytes, offset, &mut warnings);
        let role = token_role(bytes, offset, size);
        if role == "invalid_sjis" {
            warnings.push(format!("invalid CP932 pair at 0x{offset:X}"));
        }
        tokens.push(LexToken {
            index: tokens.len(),
            offset,
            size,
            role: role.to_owned(),
            raw: hex_spaced(&bytes[offset..offset + size]),
            owner_opcode_index: None,
        });
        offset += size;
    }
    (tokens, warnings)
}

fn immediate_size(bytes: &[u8], offset: usize) -> usize {
    match bytes.get(offset).copied() {
        Some(0x23..=0x27) => 1,
        Some(0x28) if offset + 1 < bytes.len() => 2,
        Some(0x29..=0x2c) if offset + 2 < bytes.len() => 3,
        _ => 0,
    }
}

fn mark_operand_range(tokens: &mut [LexToken], start: usize, end: usize, owner: usize, role: &str) {
    for token in tokens.iter_mut() {
        if token.offset < end && token.offset + token.size > start {
            token.owner_opcode_index.get_or_insert(owner);
            token.role = role.to_owned();
        }
    }
}

fn claim_exprs(
    bytes: &[u8],
    tokens: &mut [LexToken],
    owner: usize,
    cursor: &mut usize,
    maximum: usize,
) {
    for _ in 0..maximum {
        let size = immediate_size(bytes, *cursor);
        if size == 0 {
            break;
        }
        mark_operand_range(
            tokens,
            *cursor,
            cursor.saturating_add(size),
            owner,
            "handler_expression",
        );
        *cursor += size;
    }
}

fn claim_quote(bytes: &[u8], tokens: &mut [LexToken], owner: usize, cursor: &mut usize) -> bool {
    if bytes.get(*cursor) != Some(&0x22) {
        return false;
    }
    let size = bytes[*cursor + 1..]
        .iter()
        .position(|&byte| byte == 0x22)
        .map_or(bytes.len() - *cursor, |relative| relative + 2);
    mark_operand_range(
        tokens,
        *cursor,
        cursor.saturating_add(size),
        owner,
        "handler_string",
    );
    *cursor += size;
    true
}

fn annotate_handler_operands(bytes: &[u8], tokens: &mut [LexToken]) {
    let mut owner = 0;
    while owner < tokens.len() {
        if tokens[owner].role != "opcode" {
            owner += 1;
            continue;
        }
        let opcode = bytes[tokens[owner].offset];
        let mut cursor = tokens[owner].offset + tokens[owner].size;
        match opcode {
            0xa6 => claim_exprs(bytes, tokens, owner, &mut cursor, 4),
            0xa7 => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xa8 => claim_exprs(bytes, tokens, owner, &mut cursor, 3),
            0xa9 => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xaa => claim_exprs(bytes, tokens, owner, &mut cursor, 1),
            0xab | 0xac => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xad => claim_exprs(bytes, tokens, owner, &mut cursor, usize::MAX),
            0xae | 0xaf | 0xb2 => {}
            0xb0 => {
                claim_quote(bytes, tokens, owner, &mut cursor);
                claim_exprs(bytes, tokens, owner, &mut cursor, 2);
            }
            0xb1 | 0xc9 => {
                claim_quote(bytes, tokens, owner, &mut cursor);
            }
            0xb3 => mark_operand_range(
                tokens,
                cursor,
                cursor.saturating_add(1),
                owner,
                "handler_operand",
            ),
            0xb4 => mark_operand_range(
                tokens,
                cursor,
                cursor.saturating_add(2),
                owner,
                "handler_operand",
            ),
            0xb5 => {
                claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                mark_operand_range(
                    tokens,
                    cursor,
                    cursor.saturating_add(1),
                    owner,
                    "handler_operand",
                );
            }
            0xb6 => {
                if !claim_quote(bytes, tokens, owner, &mut cursor) {
                    mark_operand_range(
                        tokens,
                        cursor,
                        cursor.saturating_add(1),
                        owner,
                        "handler_operand",
                    );
                    cursor = cursor.saturating_add(1);
                    claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                }
            }
            0xb7 => claim_exprs(bytes, tokens, owner, &mut cursor, 3),
            0xb8 => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xb9 => {
                claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                mark_operand_range(
                    tokens,
                    cursor,
                    cursor.saturating_add(1),
                    owner,
                    "handler_operand",
                );
            }
            0xba | 0xbe | 0xcb | 0xd5 => claim_exprs(bytes, tokens, owner, &mut cursor, 1),
            0xbb => {
                claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                mark_operand_range(
                    tokens,
                    cursor,
                    cursor.saturating_add(1),
                    owner,
                    "handler_operand",
                );
            }
            0xbc | 0xbd => mark_operand_range(
                tokens,
                cursor,
                cursor.saturating_add(1),
                owner,
                "handler_operand",
            ),
            0xbf | 0xcc | 0xce | 0xd3 => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xc0 => {
                claim_quote(bytes, tokens, owner, &mut cursor);
            }
            0xc1 | 0xc2 => {
                claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                claim_quote(bytes, tokens, owner, &mut cursor);
            }
            0xc3..=0xc5 => claim_exprs(bytes, tokens, owner, &mut cursor, 2),
            0xc6 => claim_exprs(bytes, tokens, owner, &mut cursor, 5),
            0xc7 => claim_exprs(bytes, tokens, owner, &mut cursor, 1),
            0xc8 => {
                claim_quote(bytes, tokens, owner, &mut cursor);
                claim_exprs(bytes, tokens, owner, &mut cursor, 1);
            }
            0xca => {
                claim_exprs(bytes, tokens, owner, &mut cursor, 1);
            }
            0xcd => {
                claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                loop {
                    if claim_quote(bytes, tokens, owner, &mut cursor) {
                        continue;
                    }
                    let before = cursor;
                    claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                    if before == cursor {
                        break;
                    }
                }
            }
            0xcf => claim_exprs(bytes, tokens, owner, &mut cursor, 4),
            0xd1 => claim_exprs(bytes, tokens, owner, &mut cursor, 3),
            0xd0 => {
                if bytes
                    .get(cursor..cursor.saturating_add(2))
                    .is_some_and(|v| v.eq_ignore_ascii_case(b"se"))
                {
                    mark_operand_range(tokens, cursor, cursor + 2, owner, "handler_operand");
                    cursor += 2;
                    while bytes.get(cursor) == Some(&0x20) {
                        mark_operand_range(tokens, cursor, cursor + 1, owner, "handler_operand");
                        cursor += 1;
                    }
                    if !claim_quote(bytes, tokens, owner, &mut cursor) {
                        claim_exprs(bytes, tokens, owner, &mut cursor, 1);
                    }
                } else if !claim_quote(bytes, tokens, owner, &mut cursor) {
                    claim_exprs(bytes, tokens, owner, &mut cursor, 2);
                }
            }
            0xd2 | 0xd4 | 0xd6..=0xd8 => {}
            _ => {}
        }
        owner += 1;
    }
}

fn decode_text(bytes: &[u8]) -> Result<String> {
    let mut restored = Vec::with_capacity(bytes.len() * 2);
    let mut offset = 0;
    while offset < bytes.len() {
        if strict_sjis_pair(bytes, offset) {
            restored.extend_from_slice(&bytes[offset..offset + 2]);
            offset += 2;
        } else if (0x2d..=0x7f).contains(&bytes[offset]) {
            restored.extend_from_slice(&[0x82, bytes[offset] + 0x72]);
            offset += 1;
        } else {
            return Err(format!(
                "non-display byte {:02X} at +0x{offset:X}",
                bytes[offset]
            ));
        }
    }
    SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&restored)
        .map(|value| value.into_owned())
        .ok_or_else(|| "CP932 decode failed after custom hiragana restoration".to_owned())
}

fn split_speaker(text: &str) -> (Option<String>, String) {
    let Some(rest) = text.strip_prefix('［') else {
        return (None, text.to_owned());
    };
    let Some(closing) = rest.find('］') else {
        return (None, text.to_owned());
    };
    if closing == 0 {
        return (None, text.to_owned());
    }
    let after = closing + '］'.len_utf8();
    (Some(rest[..closing].to_owned()), rest[after..].to_owned())
}

pub fn normalize_translation(value: &str) -> Result<String> {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        let normalized = match character {
            '\0' | '\r' | '\n' => {
                return Err(
                    "message/name 不能包含 NUL、CR 或 LF；A5 换行保存在不可变脚本结构中".to_owned(),
                )
            }
            ' ' => '　',
            '!'..='~' => {
                char::from_u32(character as u32 + 0xfee0).expect("ASCII fullwidth mapping is valid")
            }
            value if value.is_control() => {
                return Err(format!("message/name 含控制字符 U+{:04X}", value as u32))
            }
            value => value,
        };
        let encoded_text = normalized.to_string();
        let (encoded, _, errors) = SHIFT_JIS.encode(&encoded_text);
        if !errors
            && encoded.len() == 2
            && SPECIAL_DBCS_CONTROLS.contains(&u16::from_be_bytes([encoded[0], encoded[1]]))
        {
            return Err(format!(
                "字符 {normalized:?} 的 CP932 编码是本引擎专用控制码，不能放入正文"
            ));
        }
        output.push(normalized);
    }
    Ok(output)
}

pub fn reserved_control_cp932() -> impl Iterator<Item = u16> {
    SPECIAL_DBCS_CONTROLS.into_iter()
}

fn composed_text(name: Option<&str>, message: &str) -> String {
    match name {
        Some(name) => format!("［{name}］{message}"),
        None => message.to_owned(),
    }
}

pub fn parse_script(bytes: &[u8], file: String) -> Result<ScriptJson> {
    let (mut tokens, warnings) = lex_full_file(bytes);
    annotate_handler_operands(bytes, &mut tokens);
    let mut entries = Vec::new();
    let mut token_index = 0;
    while token_index < tokens.len() {
        if tokens[token_index].role != "text" {
            token_index += 1;
            continue;
        }
        let token_start = token_index;
        while token_index < tokens.len() && tokens[token_index].role == "text" {
            token_index += 1;
        }
        let token_end = token_index;
        let offset = tokens[token_start].offset;
        let end = tokens[token_end - 1].offset + tokens[token_end - 1].size;
        let decoded = decode_text(&bytes[offset..end])
            .map_err(|error| format!("{file}+0x{offset:X}: {error}"))?;
        let (scr_name, scr_msg) = split_speaker(&decoded);
        let index = entries.len();
        entries.push(TextEntry {
            file: file.clone(),
            index,
            token_start,
            token_end,
            offset,
            size: end - offset,
            entry_type: if scr_name.is_some() {
                "dialogue"
            } else {
                "text"
            }
            .to_owned(),
            name: scr_name.clone(),
            scr_name,
            message: scr_msg.clone(),
            scr_msg,
        });
    }
    Ok(ScriptJson {
        format: FORMAT.to_owned(),
        file,
        source_size: bytes.len(),
        source_sha256: sha256(bytes),
        encoding: ENCODING.to_owned(),
        lexer: LEXER.to_owned(),
        handler_profile: HANDLERS.to_owned(),
        name_policy: NAME_POLICY.to_owned(),
        entries,
        tokens,
        warnings,
    })
}

fn same_immutable_entry(actual: &TextEntry, expected: &TextEntry) -> bool {
    actual.file == expected.file
        && actual.index == expected.index
        && actual.token_start == expected.token_start
        && actual.token_end == expected.token_end
        && actual.offset == expected.offset
        && actual.size == expected.size
        && actual.entry_type == expected.entry_type
        && actual.scr_name == expected.scr_name
        && actual.scr_msg == expected.scr_msg
}

pub fn validate_script(script: &ScriptJson, source: &[u8]) -> Result<ScriptJson> {
    if script.format != FORMAT
        || script.encoding != ENCODING
        || script.lexer != LEXER
        || script.handler_profile != HANDLERS
        || script.name_policy != NAME_POLICY
    {
        return Err(format!("不支持或已修改的 MES JSON 元数据；需要 {FORMAT}"));
    }
    if script.source_size != source.len() || script.source_sha256 != sha256(source) {
        return Err("MES JSON 对应的源文件大小或 SHA-256 不匹配".to_owned());
    }
    let expected = parse_script(source, script.file.clone())?;
    if script.tokens != expected.tokens || script.warnings != expected.warnings {
        return Err("_tokens/_warnings 与源 MES 不匹配".to_owned());
    }
    if script.entries.len() != expected.entries.len()
        || script
            .entries
            .iter()
            .zip(&expected.entries)
            .any(|(actual, expected)| !same_immutable_entry(actual, expected))
    {
        return Err("entries 的不可变元数据、scr_msg 或 _scr_name 已被修改".to_owned());
    }
    for entry in &script.entries {
        match (&entry.scr_name, &entry.name) {
            (Some(_), Some(name)) if !name.is_empty() => {}
            (Some(_), _) => return Err(format!("entry {} 必须保留非空 name", entry.index)),
            (None, None) => {}
            (None, Some(_)) => {
                return Err(format!("entry {} 不能给无名文本新增 name", entry.index))
            }
        }
        normalize_translation(&entry.message)
            .map_err(|error| format!("entry {} message: {error}", entry.index))?;
        if let Some(name) = &entry.name {
            normalize_translation(name)
                .map_err(|error| format!("entry {} name: {error}", entry.index))?;
        }
    }
    Ok(expected)
}

pub fn final_display_texts(script: &ScriptJson) -> Result<Vec<String>> {
    script
        .entries
        .iter()
        .map(|entry| {
            normalize_translation(&composed_text(entry.name.as_deref(), &entry.message))
                .map_err(|error| format!("{} entry {}: {error}", script.file, entry.index))
        })
        .collect()
}

pub fn source_reserved_cp932(script: &ScriptJson) -> Result<BTreeSet<u16>> {
    let mut reserved = BTreeSet::new();
    for entry in &script.entries {
        let source = composed_text(entry.scr_name.as_deref(), &entry.scr_msg);
        for character in source.chars() {
            let text = character.to_string();
            let (encoded, _, errors) = SHIFT_JIS.encode(&text);
            if errors {
                return Err(format!("源文字 {character:?} 不能编码为 CP932"));
            }
            if encoded.len() == 2 {
                reserved.insert(u16::from_be_bytes([encoded[0], encoded[1]]));
            }
        }
    }
    Ok(reserved)
}

pub fn rebuild_script(script: &ScriptJson, source: &[u8], plan: &EncodingPlan) -> Result<Vec<u8>> {
    let expected = validate_script(script, source)?;
    let mut output = Vec::with_capacity(source.len());
    let mut token_index = 0;
    let mut entry_index = 0;
    while token_index < expected.tokens.len() {
        if entry_index < script.entries.len()
            && script.entries[entry_index].token_start == token_index
        {
            let entry = &script.entries[entry_index];
            let unchanged = entry.name == entry.scr_name && entry.message == entry.scr_msg;
            if unchanged {
                output.extend_from_slice(&source[entry.offset..entry.offset + entry.size]);
            } else {
                let display =
                    normalize_translation(&composed_text(entry.name.as_deref(), &entry.message))?;
                output.extend_from_slice(&plan.encode_cp932(&display)?);
            }
            token_index = entry.token_end;
            entry_index += 1;
        } else {
            let token = &expected.tokens[token_index];
            output.extend_from_slice(&source[token.offset..token.offset + token.size]);
            token_index += 1;
        }
    }
    if entry_index != script.entries.len() {
        return Err(format!("entry {entry_index} 未在 token 流中到达"));
    }
    Ok(output)
}

fn sha256(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn hex_spaced(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_text(value: &str) -> Vec<u8> {
        let (encoded, _, errors) = SHIFT_JIS.encode(value);
        assert!(!errors);
        encoded.into_owned()
    }

    #[test]
    fn corrected_low_control_lengths_and_square_name() {
        let mut source = vec![0x10, 0x88, 0xa5];
        source.extend_from_slice(&source_text("［なつき］テスト"));
        source.extend_from_slice(&[0xff, 0xff]);
        let parsed = parse_script(&source, "A/MES/T.MES".into()).unwrap();
        assert_eq!(parsed.tokens[0].size, 2);
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].scr_name.as_deref(), Some("なつき"));
        assert_eq!(parsed.entries[0].scr_msg, "テスト");
    }

    #[test]
    fn handler_quote_is_not_extracted() {
        let mut source = vec![0xb1, 0x22];
        source.extend_from_slice(b"FILE.M");
        source.push(0x22);
        source.extend_from_slice(&source_text("表示"));
        let parsed = parse_script(&source, "A/MES/T.MES".into()).unwrap();
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].scr_msg, "表示");
    }

    #[test]
    fn ascii_normalizes_and_linebreak_is_rejected() {
        assert_eq!(normalize_translation("A 1!").unwrap(), "Ａ　１！");
        assert!(normalize_translation("a\nb").is_err());
        assert!(normalize_translation("{}").is_err());
    }

    #[test]
    fn rebuilds_chinese_with_global_carriers_and_keeps_a5() {
        let mut source = source_text("［なつき］テスト");
        source.push(0xa5);
        source.extend_from_slice(&source_text("次の行"));
        source.extend_from_slice(&[0xff, 0xff]);
        let mut script = parse_script(&source, "A/MES/T.MES".into()).unwrap();
        script.entries[0].name = Some("夏希".into());
        script.entries[0].message = "你好，这是一段变长测试。".into();
        let reserved = source_reserved_cp932(&script).unwrap();
        let texts = final_display_texts(&script).unwrap();
        let substitutions = vn_font::font_98::SubstitutionMap::embedded().unwrap();
        let plan = EncodingPlan::build(
            &substitutions,
            reserved.iter().copied(),
            texts.iter().map(String::as_str),
        )
        .unwrap();
        let rebuilt = rebuild_script(&script, &source, &plan).unwrap();
        assert!(rebuilt.len() > source.len());
        let reparsed = parse_script(&rebuilt, "A/MES/T.MES".into()).unwrap();
        assert_eq!(
            reparsed
                .tokens
                .iter()
                .filter(|token| token.role == "line_break")
                .count(),
            1
        );
        assert_eq!(reparsed.entries.len(), 2);
    }

    #[cfg(windows)]
    #[test]
    fn shared_backend_generates_np2_font_bmp_for_chinese() {
        let substitutions = vn_font::font_98::SubstitutionMap::embedded().unwrap();
        let plan = EncodingPlan::build(&substitutions, [], ["你好"]).unwrap();
        let font = vn_font::font_98::prepare_font(
            vn_font::font_98::EMBEDDED_FONT,
            &plan.requests(),
            &BTreeSet::new(),
            vn_font::font_98::FONT_FACE,
        )
        .unwrap();
        vn_font::font_98::validate_font(&font.bytes).unwrap();
        assert_eq!(font.bytes.len(), 524_350);
        assert_eq!(&font.bytes[..2], b"BM");
        assert_eq!(font.patched_glyphs, 2);
    }
}
