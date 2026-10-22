use crate::{
    analyze_script, decode_cp932, hex, is_cp932_lead, sha256_hex, FontPatchRequest, GaijiTable,
    GlyphTable, MessagePart, RelocationKind, ScriptAnalysis, StringKind, SubstitutionMap,
    TRANSLATION_FORMAT,
};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslationDocument {
    pub _format: String,
    pub _file: String,
    pub _source_sha256: String,
    pub _encoding: String,
    pub entries: Vec<TranslationEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslationEntry {
    pub _index: usize,
    pub _offset: usize,
    pub _size: usize,
    pub _type: StringKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _capture_id: Option<u16>,
    pub scr_msg: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InjectOptions {
    pub max_columns: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct InjectReport {
    pub bytes: Vec<u8>,
    pub entries: usize,
    pub changed_entries: usize,
    pub inserted_newlines: usize,
    pub dynamic_wrap_skips: usize,
    pub font_requests: Vec<FontPatchRequest>,
    pub literal_characters: BTreeSet<char>,
}

#[derive(Clone, Debug)]
struct Replacement {
    start: usize,
    end: usize,
    bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum InputToken {
    Character(char),
    Newline,
    Raw([u8; 2]),
    Gaiji([u8; 2]),
    H(Vec<u8>),
    X(u8),
}

pub fn extract_translation(
    source: &[u8],
    glyphs: &GlyphTable,
    source_file: &str,
) -> Result<TranslationDocument> {
    extract_translation_internal(source, glyphs, source_file, None)
}

pub fn extract_translation_with_gaiji(
    source: &[u8],
    glyphs: &GlyphTable,
    source_file: &str,
    gaiji: &GaijiTable,
) -> Result<TranslationDocument> {
    extract_translation_internal(source, glyphs, source_file, Some(gaiji))
}

fn extract_translation_internal(
    source: &[u8],
    glyphs: &GlyphTable,
    source_file: &str,
    gaiji: Option<&GaijiTable>,
) -> Result<TranslationDocument> {
    let analysis = analyze_script(source, glyphs, source_file)?;
    let mut entries = Vec::with_capacity(analysis.strings.len());
    for record in &analysis.strings {
        let scr_msg = translation_message(record, glyphs, gaiji)?;
        entries.push(TranslationEntry {
            _index: record._index,
            _offset: record._offset,
            _size: record._size,
            _type: record._type.clone(),
            _capture_id: record.capture_id,
            message: scr_msg.clone(),
            scr_msg,
        });
    }
    Ok(TranslationDocument {
        _format: TRANSLATION_FORMAT.to_string(),
        _file: source_file.to_string(),
        _source_sha256: analysis.source_sha256,
        _encoding: encoding_marker(gaiji).to_string(),
        entries,
    })
}

pub fn export_plain_text(document: &TranslationDocument) -> Result<Vec<u8>> {
    let mut output = String::new();
    for entry in &document.entries {
        for character in entry.message.chars() {
            match character {
                '\\' => output.push_str("\\\\"),
                '\n' => output.push_str("\\n"),
                '\r' => {
                    return Err(format!("第 {} 条包含不支持的回车字符 U+000D", entry._index));
                }
                _ => output.push(character),
            }
        }
        output.push('\n');
    }
    Ok(output.into_bytes())
}

pub fn import_plain_text(document: &mut TranslationDocument, bytes: &[u8]) -> Result<()> {
    let text =
        std::str::from_utf8(bytes).map_err(|error| format!("TXT 不是有效 UTF-8: {error}"))?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    if text.contains('\0') {
        return Err("TXT 包含 NUL 字符".to_string());
    }

    let normalized = normalize_txt_line_endings(text)?;
    let mut lines = if normalized.is_empty() {
        Vec::new()
    } else {
        normalized.split('\n').collect::<Vec<_>>()
    };
    if normalized.ends_with('\n') {
        lines.pop();
    }
    if lines.len() != document.entries.len() {
        return Err(format!(
            "TXT 行数与脚本文本条目数不匹配: TXT={} 脚本={}",
            lines.len(),
            document.entries.len()
        ));
    }

    for (line, entry) in lines.into_iter().zip(&mut document.entries) {
        entry.message = decode_plain_text_line(line)
            .map_err(|error| format!("TXT 第 {} 行: {error}", entry._index))?;
    }
    Ok(())
}

fn normalize_txt_line_endings(text: &str) -> Result<String> {
    let mut output = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\r' {
            if characters.next_if_eq(&'\n').is_none() {
                return Err("TXT 含有孤立回车符；只接受 LF 或 CRLF 行尾".to_string());
            }
            output.push('\n');
        } else {
            output.push(character);
        }
    }
    Ok(output)
}

fn decode_plain_text_line(line: &str) -> Result<String> {
    let mut output = String::with_capacity(line.len());
    let mut characters = line.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        match characters.next() {
            Some('\\') => output.push('\\'),
            Some('n') => output.push('\n'),
            Some(other) => {
                return Err(format!("不支持的转义 \\{other}；只允许 \\n 和 \\\\"));
            }
            None => return Err("行尾单独的反斜杠不是完整转义".to_string()),
        }
    }
    Ok(output)
}

pub fn inject_translation(
    source: &[u8],
    glyphs: &GlyphTable,
    source_file: &str,
    document: &TranslationDocument,
    options: InjectOptions,
) -> Result<InjectReport> {
    inject_translation_internal(source, glyphs, source_file, document, options, None, None)
}

pub fn inject_translation_localized(
    source: &[u8],
    glyphs: &GlyphTable,
    source_file: &str,
    document: &TranslationDocument,
    options: InjectOptions,
    gaiji: &GaijiTable,
    substitutions: &SubstitutionMap,
) -> Result<InjectReport> {
    inject_translation_internal(
        source,
        glyphs,
        source_file,
        document,
        options,
        Some(gaiji),
        Some(substitutions),
    )
}

fn inject_translation_internal(
    source: &[u8],
    glyphs: &GlyphTable,
    source_file: &str,
    document: &TranslationDocument,
    options: InjectOptions,
    gaiji: Option<&GaijiTable>,
    substitutions: Option<&SubstitutionMap>,
) -> Result<InjectReport> {
    if document._format != TRANSLATION_FORMAT {
        return Err(format!(
            "不支持的翻译格式 {:?}，应为 {TRANSLATION_FORMAT}",
            document._format
        ));
    }
    if document._file != source_file {
        return Err(format!(
            "翻译文件对应 {:?}，当前源文件是 {:?}",
            document._file, source_file
        ));
    }
    let source_sha256 = sha256_hex(source);
    if document._source_sha256 != source_sha256 {
        return Err(format!(
            "源文件 SHA-256 不匹配: 翻译记录={} 当前={source_sha256}",
            document._source_sha256
        ));
    }
    if document._encoding != encoding_marker(gaiji) {
        return Err(format!("不支持的文本编码标记: {}", document._encoding));
    }
    if matches!(options.max_columns, Some(0)) {
        return Err("--max-columns 必须大于 0".to_string());
    }

    let analysis = analyze_script(source, glyphs, source_file)?;
    if document.entries.len() != analysis.strings.len() {
        return Err(format!(
            "条目数不匹配: 翻译={} 源脚本={}",
            document.entries.len(),
            analysis.strings.len()
        ));
    }

    let mut replacements = Vec::with_capacity(document.entries.len());
    let mut changed_entries = 0usize;
    let mut inserted_newlines = 0usize;
    let mut dynamic_wrap_skips = 0usize;
    let mut font_requests = Vec::new();
    let mut literal_characters = BTreeSet::new();
    for (entry, record) in document.entries.iter().zip(&analysis.strings) {
        if entry._index != record._index
            || entry._offset != record._offset
            || entry._size != record._size
            || entry._type != record._type
            || entry._capture_id != record.capture_id
        {
            return Err(format!("第 {} 条的定位元数据与源脚本不一致", record._index));
        }
        let current_scr_msg = translation_message(record, glyphs, gaiji)?;
        if entry.scr_msg != current_scr_msg {
            return Err(format!(
                "第 {} 条 scr_msg 与源脚本不一致，拒绝按旧文本注入",
                record._index
            ));
        }
        let source_tokens = parse_message(&entry.scr_msg, gaiji)?;
        let message_tokens = parse_message(&entry.message, gaiji)
            .map_err(|error| format!("第 {} 条 message: {error}", record._index))?;
        if structural_tokens(&source_tokens) != structural_tokens(&message_tokens) {
            return Err(format!(
                "第 {} 条的 H/X/RAW/GAIJI 结构被增删、改值或调换顺序",
                record._index
            ));
        }

        let encoded = if substitutions.is_some()
            || entry.message != entry.scr_msg
            || options.max_columns.is_some()
        {
            Some(
                encode_tokens(&message_tokens, glyphs, options.max_columns, substitutions)
                    .map_err(|error| format!("第 {} 条无法编码: {error}", record._index))?,
            )
        } else {
            None
        };
        let (replacement, added, skipped_dynamic) =
            if entry.message == entry.scr_msg && options.max_columns.is_none() {
                (source[record._offset..record._end].to_vec(), 0, false)
            } else {
                let encoded = encoded.as_ref().expect("changed text was encoded");
                (
                    encoded.bytes.clone(),
                    encoded.inserted_newlines,
                    encoded.skipped_dynamic,
                )
            };
        if let Some(encoded) = encoded {
            font_requests.extend(encoded.font_requests);
            literal_characters.extend(encoded.literal_characters);
        }
        if replacement != source[record._offset..record._end] {
            changed_entries += 1;
        }
        inserted_newlines += added;
        dynamic_wrap_skips += usize::from(skipped_dynamic);
        replacements.push(Replacement {
            start: record._offset,
            end: record._end,
            bytes: replacement,
        });
    }

    let bytes = rebuild_script(source, &analysis, &replacements)?;
    let rebuilt = analyze_script(&bytes, glyphs, source_file)
        .map_err(|error| format!("重建后的脚本未通过结构复核: {error}"))?;
    let expected_terminator = AddressMap::new(&replacements).map(
        analysis
            .terminator_offset
            .ok_or_else(|| "源脚本没有 0x1A 终止符".to_string())?,
    )?;
    if rebuilt.terminator_offset != Some(expected_terminator) {
        return Err("重建后的终止符位置不符合重定位结果".to_string());
    }

    Ok(InjectReport {
        bytes,
        entries: document.entries.len(),
        changed_entries,
        inserted_newlines,
        dynamic_wrap_skips,
        font_requests,
        literal_characters,
    })
}

fn encoding_marker(gaiji: Option<&GaijiTable>) -> &'static str {
    if gaiji.is_some() {
        "cp932+adv-a0-df+gaiji"
    } else {
        "cp932+adv-a0-df"
    }
}

fn translation_message(
    record: &crate::StringRecord,
    glyphs: &GlyphTable,
    gaiji: Option<&GaijiTable>,
) -> Result<String> {
    let mut output = String::new();
    for part in &record.parts {
        match part {
            MessagePart::Text { raw_hex, .. } => {
                let raw = decode_hex(raw_hex)?;
                output.push_str(&decode_translation_text(&raw, glyphs, gaiji)?);
            }
            MessagePart::Newline { .. } => output.push('\n'),
            MessagePart::InlineCommand {
                opcode, raw_hex, ..
            } if opcode == "48" => {
                output.push_str("{{H:");
                output.push_str(raw_hex);
                output.push_str("}}");
            }
            MessagePart::InlineCommand {
                opcode, raw_hex, ..
            } if opcode == "58" => {
                let raw = decode_hex(raw_hex)?;
                if raw.len() != 2 || raw[0] != b'X' {
                    return Err(format!("无效 X 内嵌控制: {raw_hex}"));
                }
                output.push_str(&format!("{{{{X:{:02X}}}}}", raw[1]));
            }
            MessagePart::InlineCommand { opcode, .. } => {
                return Err(format!("不支持的内嵌控制 0x{opcode}"));
            }
        }
    }
    Ok(output)
}

fn decode_translation_text(
    raw: &[u8],
    glyphs: &GlyphTable,
    gaiji: Option<&GaijiTable>,
) -> Result<String> {
    let mut output = String::new();
    let mut pos = 0usize;
    while pos < raw.len() {
        let byte = raw[pos];
        if byte == 0x20 || (0xA0..=0xDF).contains(&byte) {
            output.push_str(glyphs.decode_single(byte)?);
            pos += 1;
        } else if is_cp932_lead(byte) {
            let pair = raw
                .get(pos..pos + 2)
                .ok_or_else(|| format!("文本尾部缺少 CP932 跟随字节: 0x{byte:02X}"))?;
            let pair_array = [pair[0], pair[1]];
            if let Some(display) = gaiji.and_then(|table| table.decode_pair(pair_array)) {
                output.push_str(display);
            } else {
                match decode_cp932(pair) {
                    Ok(text) => output.push_str(&text),
                    Err(_) => output.push_str(&format!("{{{{RAW:{}}}}}", hex(pair))),
                }
            }
            pos += 2;
        } else {
            return Err(format!("文本中出现非文本字节 0x{byte:02X}"));
        }
    }
    Ok(output)
}

fn parse_message(message: &str, gaiji: Option<&GaijiTable>) -> Result<Vec<InputToken>> {
    let mut tokens = Vec::new();
    let mut offset = 0usize;
    while offset < message.len() {
        let rest = &message[offset..];
        if rest.starts_with("{{") {
            let close = rest
                .find("}}")
                .ok_or_else(|| format!("UTF-8 offset {offset}: 占位符缺少 '}}}}'"))?;
            let body = &rest[2..close];
            let token = if let Some(value) = body.strip_prefix("RAW:") {
                let raw = decode_hex_exact(value, 2, "RAW")?;
                InputToken::Raw([raw[0], raw[1]])
            } else if let Some(value) = body.strip_prefix("H:") {
                let raw = decode_hex(value)?;
                if raw.len() < 4 || raw[0] != b'H' || raw.last() != Some(&0x7F) {
                    return Err(format!("无效 H 占位符: {{{{{body}}}}}"));
                }
                InputToken::H(raw)
            } else if let Some(value) = body.strip_prefix("X:") {
                let raw = decode_hex_exact(value, 1, "X")?;
                InputToken::X(raw[0])
            } else {
                return Err(format!("未知占位符: {{{{{body}}}}}"));
            };
            tokens.push(token);
            offset += close + 2;
            continue;
        }
        if let Some((pair, consumed)) = gaiji.and_then(|table| table.match_display(rest)) {
            tokens.push(InputToken::Gaiji(pair));
            offset += consumed;
            continue;
        }
        let character = rest.chars().next().expect("non-empty remainder");
        match character {
            '\n' => tokens.push(InputToken::Newline),
            '\r' => return Err("不接受 CR；JSON 中请统一使用 \\n".to_string()),
            '\0' => return Err("文本不能包含 NUL".to_string()),
            _ => tokens.push(InputToken::Character(character)),
        }
        offset += character.len_utf8();
    }
    Ok(tokens)
}

fn structural_tokens(tokens: &[InputToken]) -> Vec<String> {
    tokens
        .iter()
        .filter_map(|token| match token {
            InputToken::Raw(raw) => Some(format!("RAW:{}", hex(raw))),
            InputToken::Gaiji(raw) => Some(format!("GAIJI:{}", hex(raw))),
            InputToken::H(raw) => Some(format!("H:{}", hex(raw))),
            InputToken::X(slot) => Some(format!("X:{slot:02X}")),
            InputToken::Character(_) | InputToken::Newline => None,
        })
        .collect()
}

struct EncodedTokens {
    bytes: Vec<u8>,
    inserted_newlines: usize,
    skipped_dynamic: bool,
    font_requests: Vec<FontPatchRequest>,
    literal_characters: BTreeSet<char>,
}

fn encode_tokens(
    tokens: &[InputToken],
    glyphs: &GlyphTable,
    max_columns: Option<usize>,
    substitutions: Option<&SubstitutionMap>,
) -> Result<EncodedTokens> {
    let has_dynamic = tokens
        .iter()
        .any(|token| matches!(token, InputToken::H(_) | InputToken::X(_)));
    let wrap_at = if has_dynamic { None } else { max_columns };
    let skipped_dynamic = has_dynamic && max_columns.is_some();
    let mut output = Vec::new();
    let mut column = 0usize;
    let mut inserted = 0usize;
    let mut font_requests = Vec::new();
    let mut literal_characters = BTreeSet::new();
    for token in tokens {
        let visible = matches!(
            token,
            InputToken::Character(_) | InputToken::Raw(_) | InputToken::Gaiji(_)
        );
        if visible && matches!(wrap_at, Some(limit) if column >= limit) {
            output.push(b'R');
            column = 0;
            inserted += 1;
        }
        match token {
            InputToken::Character(character) => {
                output.extend_from_slice(&encode_character(
                    *character,
                    glyphs,
                    substitutions,
                    &mut font_requests,
                    &mut literal_characters,
                )?);
                column += 1;
            }
            InputToken::Newline => {
                output.push(b'R');
                column = 0;
            }
            InputToken::Raw(raw) => {
                output.extend_from_slice(raw);
                column += 1;
            }
            InputToken::Gaiji(raw) => {
                output.extend_from_slice(raw);
                column += 1;
            }
            InputToken::H(raw) => output.extend_from_slice(raw),
            InputToken::X(slot) => output.extend_from_slice(&[b'X', *slot]),
        }
    }
    Ok(EncodedTokens {
        bytes: output,
        inserted_newlines: inserted,
        skipped_dynamic,
        font_requests,
        literal_characters,
    })
}

fn encode_character(
    character: char,
    glyphs: &GlyphTable,
    substitutions: Option<&SubstitutionMap>,
    font_requests: &mut Vec<FontPatchRequest>,
    literal_characters: &mut BTreeSet<char>,
) -> Result<Vec<u8>> {
    if character == ' ' || character == '\u{3000}' {
        return Ok(vec![0x20]);
    }
    let text = character.to_string();
    if let Some(index) = glyphs.decoded.iter().position(|value| value == &text) {
        return Ok(vec![0xA0 + index as u8]);
    }
    let normalized = if ('!'..='~').contains(&character) {
        char::from_u32(0xFF01 + u32::from(character) - u32::from('!'))
            .expect("fullwidth ASCII mapping")
    } else {
        character
    };
    let normalized_text = normalized.to_string();
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&normalized_text);
    if !had_errors {
        let bytes = encoded.into_owned();
        if bytes.len() == 2 && is_cp932_lead(bytes[0]) {
            if substitutions.is_none() || crate::has_loaded_np2_slot(normalized) {
                if substitutions.is_some() {
                    literal_characters.insert(normalized);
                }
                return Ok(bytes);
            }
        } else if substitutions.is_none() {
            return Err(format!(
                "字符 {:?} 编码为 {}，会被解释器当作指令而非正文",
                character,
                hex(&bytes)
            ));
        }
    }

    let Some(substitutions) = substitutions else {
        return Err(format!(
            "字符 {:?} 不在当前 CP932/ADV 字形表内；需要先确定中文字库映射",
            character
        ));
    };
    let carrier = substitutions
        .carrier_for(character, normalized)
        .ok_or_else(|| format!("字符 {:?} 不在内置中文替换映射中", character))?;
    let carrier_text = carrier.to_string();
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&carrier_text);
    if had_errors || encoded.len() != 2 || !is_cp932_lead(encoded[0]) {
        return Err(format!(
            "字符 {:?} 的内置载体 {carrier} 不是双字节 CP932",
            character
        ));
    }
    if !crate::has_loaded_np2_slot(carrier) {
        return Err(format!(
            "字符 {:?} 的载体 {carrier} 不在 NP2 已加载字库页",
            character
        ));
    }
    font_requests.push(FontPatchRequest {
        carrier,
        replacement: character,
    });
    Ok(encoded.into_owned())
}

fn rebuild_script(
    source: &[u8],
    analysis: &ScriptAnalysis,
    replacements: &[Replacement],
) -> Result<Vec<u8>> {
    let terminator = analysis
        .terminator_offset
        .ok_or_else(|| "源脚本没有 0x1A 终止符，不能安全重建".to_string())?;
    if source.get(terminator) != Some(&0x1A) {
        return Err("解析记录的终止符与源字节不一致".to_string());
    }
    if source[terminator + 1..].iter().any(|byte| *byte != 0) {
        return Err("0x1A 后存在非零未知数据，拒绝重排".to_string());
    }
    let mut previous_end = 2usize;
    for replacement in replacements {
        if replacement.start < previous_end || replacement.end < replacement.start {
            return Err("文本范围交叠或逆序".to_string());
        }
        if replacement.end > terminator {
            return Err("文本范围越过 0x1A 终止符".to_string());
        }
        previous_end = replacement.end;
    }

    let address_map = AddressMap::new(replacements);
    let mut rebuilt = Vec::new();
    let mut cursor = 0usize;
    for replacement in replacements {
        rebuilt.extend_from_slice(&source[cursor..replacement.start]);
        rebuilt.extend_from_slice(&replacement.bytes);
        cursor = replacement.end;
    }
    rebuilt.extend_from_slice(&source[cursor..=terminator]);

    for relocation in &analysis.relocations {
        let field_offset = address_map.map(relocation.field_offset)?;
        let target_offset = address_map.map(relocation.target_offset).map_err(|error| {
            format!(
                "无法重定位 0x{:X} 处指向 0x{:X} 的地址: {error}",
                relocation.field_offset, relocation.target_offset
            )
        })?;
        match relocation.kind {
            RelocationKind::HeaderU16 | RelocationKind::AbsoluteU16 => {
                let value = u16::try_from(target_offset).map_err(|_| {
                    format!("重定位目标 0x{target_offset:X} 超过 16 位脚本地址范围")
                })?;
                write_at(&mut rebuilt, field_offset, &value.to_le_bytes())?;
            }
            RelocationKind::RelativeU8 => {
                let old_base = relocation.base_offset.expect("relative base");
                let base_offset = address_map.map(old_base)?;
                let delta = target_offset.checked_sub(base_offset).ok_or_else(|| {
                    format!(
                        "相对跳转 0x{:X} 重定位后变成向后跳转",
                        relocation.field_offset
                    )
                })?;
                let value = u8::try_from(delta).map_err(|_| {
                    format!(
                        "相对跳转 0x{:X} 重定位后距离 {delta} 超过 255",
                        relocation.field_offset
                    )
                })?;
                write_at(&mut rebuilt, field_offset, &[value])?;
            }
        }
    }

    let minimum_size = source.len();
    let output_size = if rebuilt.len() <= minimum_size {
        minimum_size
    } else {
        rebuilt
            .len()
            .checked_add(0xFF)
            .map(|value| value & !0xFF)
            .ok_or_else(|| "输出补齐长度溢出".to_string())?
    };
    rebuilt.resize(output_size, 0);
    Ok(rebuilt)
}

struct AddressMap<'a> {
    replacements: &'a [Replacement],
}

impl<'a> AddressMap<'a> {
    fn new(replacements: &'a [Replacement]) -> Self {
        Self { replacements }
    }

    fn map(&self, old_offset: usize) -> Result<usize> {
        let mut delta = 0i64;
        for replacement in self.replacements {
            let new_start = apply_delta(replacement.start, delta)?;
            if old_offset < replacement.start {
                break;
            }
            if old_offset == replacement.start {
                return Ok(new_start);
            }
            if old_offset < replacement.end {
                if replacement.bytes.len() == replacement.end - replacement.start {
                    return apply_delta(old_offset, delta);
                }
                return Err(format!(
                    "目标落在可变文本 0x{:X}..0x{:X} 内部",
                    replacement.start, replacement.end
                ));
            }
            delta += replacement.bytes.len() as i64 - (replacement.end - replacement.start) as i64;
            if old_offset == replacement.end {
                return apply_delta(old_offset, delta);
            }
        }
        apply_delta(old_offset, delta)
    }
}

fn apply_delta(offset: usize, delta: i64) -> Result<usize> {
    let value = offset as i128 + i128::from(delta);
    usize::try_from(value).map_err(|_| "重定位结果超出地址范围".to_string())
}

fn write_at(output: &mut [u8], offset: usize, value: &[u8]) -> Result<()> {
    let end = offset
        .checked_add(value.len())
        .ok_or_else(|| "写入地址溢出".to_string())?;
    let destination = output
        .get_mut(offset..end)
        .ok_or_else(|| format!("重定位字段 0x{offset:X} 超出输出脚本"))?;
    destination.copy_from_slice(value);
    Ok(())
}

fn decode_hex_exact(value: &str, bytes: usize, kind: &str) -> Result<Vec<u8>> {
    if value.len() != bytes * 2 {
        return Err(format!("{kind} 占位符必须是 {} 位十六进制", bytes * 2));
    }
    decode_hex(value)
}

fn decode_hex(value: &str) -> Result<Vec<u8>> {
    if !value.len().is_multiple_of(2) || value.is_empty() {
        return Err(format!("十六进制字节串长度无效: {value:?}"));
    }
    let mut output = Vec::with_capacity(value.len() / 2);
    for offset in (0..value.len()).step_by(2) {
        let byte = u8::from_str_radix(&value[offset..offset + 2], 16)
            .map_err(|_| format!("无效十六进制字节: {:?}", &value[offset..offset + 2]))?;
        output.push(byte);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glyph_table() -> GlyphTable {
        let mut decoded = Vec::new();
        for index in 0..crate::GLYPH_COUNT {
            decoded.push(char::from_u32(0xE000 + index as u32).unwrap().to_string());
        }
        GlyphTable {
            decoded,
            raw_sha256: "TEST".to_string(),
        }
    }

    #[test]
    fn unchanged_document_is_byte_exact() {
        let source = [9, 0, 0x82, 0xA0, b'R', 0xEC, 0x5B, b'A', 0x1A, 0, 0];
        let glyphs = glyph_table();
        let document = extract_translation(&source, &glyphs, "sample.bin").unwrap();
        assert!(document.entries[0].scr_msg.contains("{{RAW:EC5B}}"));
        let report = inject_translation(
            &source,
            &glyphs,
            "sample.bin",
            &document,
            InjectOptions::default(),
        )
        .unwrap();
        assert_eq!(report.bytes, source);
    }

    #[test]
    fn gaiji_is_pure_text_and_round_trips() {
        let source = [9, 0, 0x82, 0xA0, b'R', 0xEB, 0xA7, b'A', 0x1A, 0, 0];
        let glyphs = glyph_table();
        let gaiji = GaijiTable::builtin().unwrap();
        let substitutions = SubstitutionMap::embedded().unwrap();
        let document =
            extract_translation_with_gaiji(&source, &glyphs, "sample.bin", &gaiji).unwrap();
        assert!(document.entries[0].scr_msg.contains('┃'));
        assert!(!document.entries[0].scr_msg.contains("RAW:EBA7"));
        let report = inject_translation_localized(
            &source,
            &glyphs,
            "sample.bin",
            &document,
            InjectOptions::default(),
            &gaiji,
            &substitutions,
        )
        .unwrap();
        assert_eq!(report.bytes, source);
    }

    #[test]
    fn localized_chinese_uses_embedded_carrier() {
        let source = [7, 0, 0x82, 0xA0, 0x82, 0xA2, 0x1A, 0];
        let glyphs = glyph_table();
        let gaiji = GaijiTable::builtin().unwrap();
        let substitutions = SubstitutionMap::embedded().unwrap();
        let mut document =
            extract_translation_with_gaiji(&source, &glyphs, "sample.bin", &gaiji).unwrap();
        document.entries[0].message = "你".to_string();
        let report = inject_translation_localized(
            &source,
            &glyphs,
            "sample.bin",
            &document,
            InjectOptions::default(),
            &gaiji,
            &substitutions,
        )
        .unwrap();
        assert_eq!(
            report.font_requests,
            vec![FontPatchRequest {
                carrier: '凜',
                replacement: '你'
            }]
        );
        assert_eq!(report.changed_entries, 1);
    }

    #[test]
    fn rejects_changed_structural_placeholder() {
        let source = [7, 0, 0x82, 0xA0, 0xEC, 0x5B, 0x1A, 0];
        let glyphs = glyph_table();
        let mut document = extract_translation(&source, &glyphs, "sample.bin").unwrap();
        document.entries[0].message = "変更".to_string();
        let error = inject_translation(
            &source,
            &glyphs,
            "sample.bin",
            &document,
            InjectOptions::default(),
        )
        .unwrap_err();
        assert!(error.contains("结构"));
    }

    #[test]
    fn plain_text_export_has_no_prefix_and_keeps_one_record_per_line() {
        let document = TranslationDocument {
            _format: TRANSLATION_FORMAT.to_string(),
            _file: "sample.bin".to_string(),
            _source_sha256: "TEST".to_string(),
            _encoding: "cp932+adv-a0-df".to_string(),
            entries: vec![
                TranslationEntry {
                    _index: 1,
                    _offset: 10,
                    _size: 20,
                    _type: StringKind::DisplayText,
                    _capture_id: None,
                    scr_msg: String::new(),
                    message: "　前\\後\n{{RAW:EBA7}}".to_string(),
                },
                TranslationEntry {
                    _index: 2,
                    _offset: 30,
                    _size: 0,
                    _type: StringKind::MenuText,
                    _capture_id: None,
                    scr_msg: String::new(),
                    message: String::new(),
                },
            ],
        };
        let text = String::from_utf8(export_plain_text(&document).unwrap()).unwrap();
        assert_eq!(text, "　前\\\\後\\n{{RAW:EBA7}}\n\n");

        let mut imported = document.clone();
        for entry in &mut imported.entries {
            entry.message = "被替换".to_string();
        }
        import_plain_text(&mut imported, text.as_bytes()).unwrap();
        assert_eq!(imported.entries[0].message, document.entries[0].message);
        assert_eq!(imported.entries[1].message, document.entries[1].message);
    }

    #[test]
    fn plain_text_import_accepts_crlf_and_rejects_bad_records() {
        let mut document = TranslationDocument {
            _format: TRANSLATION_FORMAT.to_string(),
            _file: "sample.bin".to_string(),
            _source_sha256: "TEST".to_string(),
            _encoding: "cp932+adv-a0-df".to_string(),
            entries: vec![TranslationEntry {
                _index: 1,
                _offset: 10,
                _size: 20,
                _type: StringKind::DisplayText,
                _capture_id: None,
                scr_msg: "原文".to_string(),
                message: "原文".to_string(),
            }],
        };
        import_plain_text(&mut document, b"line\\nnext\r\n").unwrap();
        assert_eq!(document.entries[0].message, "line\nnext");

        assert!(import_plain_text(&mut document, b"one\ntwo\n")
            .unwrap_err()
            .contains("行数"));
        assert!(import_plain_text(&mut document, b"bad\\q\n")
            .unwrap_err()
            .contains("不支持的转义"));
    }

    #[test]
    fn growing_text_relocates_absolute_and_relative_targets() {
        let source = [9, 0, b'}', 0x40, 0x7F, 3, 0x82, 0xA0, b'A', 0x1A, 0, 0];
        let glyphs = glyph_table();
        let mut document = extract_translation(&source, &glyphs, "sample.bin").unwrap();
        document.entries[0].message = "いい".to_string();
        let report = inject_translation(
            &source,
            &glyphs,
            "sample.bin",
            &document,
            InjectOptions::default(),
        )
        .unwrap();
        assert_eq!(u16::from_le_bytes([report.bytes[0], report.bytes[1]]), 11);
        assert_eq!(report.bytes[5], 5);
        assert_eq!(report.bytes[11], 0x1A);
    }
}
