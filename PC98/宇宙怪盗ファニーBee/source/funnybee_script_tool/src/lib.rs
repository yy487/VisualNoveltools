use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

mod font;
mod gaiji;
mod recursive;
mod translation;

pub use font::{
    embedded_font_sha256, has_loaded_np2_slot, prepare_font, FontBuild, FontPatchRequest,
    SubstitutionMap,
};
pub use gaiji::GaijiTable;

pub use recursive::{
    recursive_extract, recursive_extract_txt, recursive_pack, recursive_pack_txt,
    RecursiveExtractReport, RecursivePackOptions, RecursivePackReport,
};
pub use translation::{
    export_plain_text, extract_translation, extract_translation_with_gaiji, import_plain_text,
    inject_translation, inject_translation_localized, InjectOptions, InjectReport,
    TranslationDocument, TranslationEntry,
};

pub const IR_FORMAT: &str = "funnybee-system3-script-analysis-v1";
pub const TRANSLATION_FORMAT: &str = "funnybee-system3-translation-v1";
pub const FUNNYBEE_GLYPH_TABLE_SHA256: &str =
    "78632CAEC99C057B8CC5FB5E77B2A2C798D7613A2BDEC36B28E5B1591EF6781E";
pub const GLYPH_TABLE_FILE_OFFSET: usize = 0x1FA8;
const GLYPH_COUNT: usize = 0x40;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug)]
pub struct GlyphTable {
    decoded: Vec<String>,
    pub raw_sha256: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScriptAnalysis {
    pub _format: String,
    pub source_file: String,
    pub source_bytes: usize,
    pub source_sha256: String,
    pub initial_word: u16,
    pub parsed_bytes: usize,
    pub terminator_offset: Option<usize>,
    pub opaque_tail_offset: usize,
    pub opaque_tail_size: usize,
    pub opaque_tail_sha256: String,
    pub glyph_table_sha256: String,
    pub command_counts: BTreeMap<String, usize>,
    pub relocations: Vec<Relocation>,
    pub strings: Vec<StringRecord>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Relocation {
    pub field_offset: usize,
    pub target_offset: usize,
    pub kind: RelocationKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_offset: Option<usize>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelocationKind {
    HeaderU16,
    AbsoluteU16,
    RelativeU8,
}

#[derive(Clone, Debug, Serialize)]
pub struct StringRecord {
    pub _index: usize,
    pub _offset: usize,
    pub _end: usize,
    pub _size: usize,
    pub _type: StringKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_id: Option<u16>,
    pub scr_msg: String,
    pub parts: Vec<MessagePart>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StringKind {
    DisplayText,
    CaptureText,
    MenuText,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MessagePart {
    Text {
        offset: usize,
        size: usize,
        raw_hex: String,
        text: String,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        unmapped_glyphs: Vec<RawGlyph>,
    },
    Newline {
        offset: usize,
        opcode: String,
    },
    InlineCommand {
        offset: usize,
        size: usize,
        opcode: String,
        raw_hex: String,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct RawGlyph {
    pub offset: usize,
    pub raw_hex: String,
    pub placeholder: String,
}

#[derive(Debug)]
struct PendingString {
    start: usize,
    end: usize,
    kind: StringKind,
    capture_id: Option<u16>,
    scr_msg: String,
    parts: Vec<MessagePart>,
}

impl PendingString {
    fn new(start: usize, kind: StringKind, capture_id: Option<u16>) -> Self {
        Self {
            start,
            end: start,
            kind,
            capture_id,
            scr_msg: String::new(),
            parts: Vec::new(),
        }
    }

    fn add_text(
        &mut self,
        offset: usize,
        raw: &[u8],
        text: String,
        unmapped_glyphs: Vec<RawGlyph>,
    ) {
        self.end = offset + raw.len();
        self.scr_msg.push_str(&text);
        self.parts.push(MessagePart::Text {
            offset,
            size: raw.len(),
            raw_hex: hex(raw),
            text,
            unmapped_glyphs,
        });
    }

    fn add_newline(&mut self, offset: usize) {
        self.end = offset + 1;
        self.scr_msg.push('\n');
        self.parts.push(MessagePart::Newline {
            offset,
            opcode: "52".to_string(),
        });
    }

    fn add_inline_command(&mut self, offset: usize, raw: &[u8], opcode: u8) {
        self.end = offset + raw.len();
        self.parts.push(MessagePart::InlineCommand {
            offset,
            size: raw.len(),
            opcode: format!("{:02X}", opcode),
            raw_hex: hex(raw),
        });
    }

    fn has_content(&self) -> bool {
        !self.parts.is_empty()
    }
}

#[derive(Clone, Copy, Debug)]
struct Expression {
    value: Option<u16>,
}

impl GlyphTable {
    pub fn from_adv(adv: &[u8]) -> Result<Self> {
        let table_size = GLYPH_COUNT * 2;
        let end = GLYPH_TABLE_FILE_OFFSET
            .checked_add(table_size)
            .ok_or_else(|| "字形表范围溢出".to_string())?;
        let raw = adv.get(GLYPH_TABLE_FILE_OFFSET..end).ok_or_else(|| {
            format!("ADV.COM 过小，无法读取 0x{GLYPH_TABLE_FILE_OFFSET:X}..0x{end:X} 字形表")
        })?;
        let mut decoded = Vec::with_capacity(GLYPH_COUNT);
        for index in 0..GLYPH_COUNT {
            let little_endian = &raw[index * 2..index * 2 + 2];
            let cp932 = [little_endian[1], little_endian[0]];
            decoded.push(
                decode_cp932(&cp932)
                    .map_err(|error| format!("字形表 0x{:02X}: {error}", 0xA0 + index))?,
            );
        }
        Ok(Self {
            decoded,
            raw_sha256: sha256_hex(raw),
        })
    }

    pub fn decode_single(&self, byte: u8) -> Result<&str> {
        let mapped = if byte == 0x20 { 0xA0 } else { byte };
        if !(0xA0..=0xDF).contains(&mapped) {
            return Err(format!("0x{byte:02X} 不是引擎单字节字形"));
        }
        Ok(&self.decoded[usize::from(mapped - 0xA0)])
    }
}

pub fn analyze_file(input: &Path, adv_path: &Path) -> Result<ScriptAnalysis> {
    let bytes =
        fs::read(input).map_err(|error| format!("无法读取 {}: {error}", input.display()))?;
    let adv =
        fs::read(adv_path).map_err(|error| format!("无法读取 {}: {error}", adv_path.display()))?;
    let glyphs = GlyphTable::from_adv(&adv)?;
    let source_name = input
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("输入文件名无法表示为 Unicode: {}", input.display()))?;
    analyze_script(&bytes, &glyphs, source_name)
}

pub fn analyze_script(
    bytes: &[u8],
    glyphs: &GlyphTable,
    source_file: &str,
) -> Result<ScriptAnalysis> {
    if bytes.len() < 3 {
        return Err("脚本成员少于 3 字节".to_string());
    }
    let initial_word = u16::from_le_bytes([bytes[0], bytes[1]]);
    let mut parser = Parser {
        bytes,
        glyphs,
        pos: 2,
        capture_mode: false,
        capture_id: None,
        menu_mode: None,
        pending: None,
        strings: Vec::new(),
        command_counts: BTreeMap::new(),
        relocations: vec![Relocation {
            field_offset: 0,
            target_offset: usize::from(initial_word),
            kind: RelocationKind::HeaderU16,
            base_offset: None,
        }],
        terminator_offset: None,
    };
    parser.parse()?;
    parser.flush_pending();
    let tail_offset = parser.pos;
    let tail = &bytes[tail_offset..];
    Ok(ScriptAnalysis {
        _format: IR_FORMAT.to_string(),
        source_file: source_file.to_string(),
        source_bytes: bytes.len(),
        source_sha256: sha256_hex(bytes),
        initial_word,
        parsed_bytes: tail_offset,
        terminator_offset: parser.terminator_offset,
        opaque_tail_offset: tail_offset,
        opaque_tail_size: tail.len(),
        opaque_tail_sha256: sha256_hex(tail),
        glyph_table_sha256: glyphs.raw_sha256.clone(),
        command_counts: parser.command_counts,
        relocations: parser.relocations,
        strings: parser.strings,
    })
}

struct Parser<'a> {
    bytes: &'a [u8],
    glyphs: &'a GlyphTable,
    pos: usize,
    capture_mode: bool,
    capture_id: Option<u16>,
    menu_mode: Option<u16>,
    pending: Option<PendingString>,
    strings: Vec<StringRecord>,
    command_counts: BTreeMap<String, usize>,
    relocations: Vec<Relocation>,
    terminator_offset: Option<usize>,
}

impl Parser<'_> {
    fn parse(&mut self) -> Result<()> {
        while self.pos < self.bytes.len() {
            if self.bytes[self.pos] == 0x1A {
                self.flush_pending();
                self.terminator_offset = Some(self.pos);
                self.pos += 1;
                return Ok(());
            }
            if self.remaining_is_zero() {
                self.flush_pending();
                return Ok(());
            }
            let byte = self.bytes[self.pos];
            if is_text_start(byte) {
                self.parse_text_run()?;
                continue;
            }
            self.parse_command(byte)?;
        }
        Ok(())
    }

    fn parse_text_run(&mut self) -> Result<()> {
        let start = self.pos;
        let mut text = String::new();
        let mut unmapped_glyphs = Vec::new();
        while self.pos < self.bytes.len() && is_text_start(self.bytes[self.pos]) {
            let byte = self.bytes[self.pos];
            if byte == 0x20 || (0xA0..=0xDF).contains(&byte) {
                text.push_str(self.glyphs.decode_single(byte)?);
                self.pos += 1;
            } else {
                let pair_offset = self.pos;
                let pair = self.take(2, "CP932 双字节字符")?;
                match decode_cp932(pair) {
                    Ok(decoded) => text.push_str(&decoded),
                    Err(_) => {
                        let raw_hex = hex(pair);
                        let placeholder = format!("⟦{raw_hex}⟧");
                        text.push_str(&placeholder);
                        unmapped_glyphs.push(RawGlyph {
                            offset: pair_offset,
                            raw_hex,
                            placeholder,
                        });
                    }
                }
            }
        }
        let raw = &self.bytes[start..self.pos];
        let kind = if self.capture_mode {
            StringKind::CaptureText
        } else {
            StringKind::DisplayText
        };
        self.ensure_pending(start, kind, self.capture_id);
        self.pending
            .as_mut()
            .expect("pending exists")
            .add_text(start, raw, text, unmapped_glyphs);
        Ok(())
    }

    fn parse_command(&mut self, opcode: u8) -> Result<()> {
        let start = self.pos;
        self.pos += 1;
        *self
            .command_counts
            .entry(format!("{:02X}", opcode))
            .or_default() += 1;
        match opcode {
            b'!' => {
                self.flush_before_command();
                self.skip_variable_selector()?;
                self.parse_expression()?;
            }
            b'{' => {
                self.flush_before_command();
                self.parse_expression()?;
                self.take_absolute_u16("{ 跳转地址")?;
            }
            b'}' => {
                self.flush_before_command();
                self.parse_expression()?;
                let field_offset = self.pos;
                let delta = usize::from(self.take(1, "} 相对偏移")?[0]);
                let base_offset = self.pos;
                let target_offset = base_offset
                    .checked_add(delta)
                    .ok_or_else(|| format!("offset 0x{field_offset:X}: 相对跳转目标溢出"))?;
                self.relocations.push(Relocation {
                    field_offset,
                    target_offset,
                    kind: RelocationKind::RelativeU8,
                    base_offset: Some(base_offset),
                });
            }
            b'@' | b'\\' => {
                self.flush_before_command();
                self.take_absolute_u16("绝对地址")?;
            }
            b']' | b'F' => self.flush_before_command(),
            b'P' => {
                self.flush_before_command();
                self.parse_expressions(4)?;
            }
            b'R' => {
                if self.capture_mode {
                    return Err(format!("offset 0x{start:X}: 捕获字符串中出现 R"));
                }
                self.ensure_pending(start, StringKind::DisplayText, None);
                self.pending
                    .as_mut()
                    .expect("pending exists")
                    .add_newline(start);
            }
            b'%' | b'G' | b'&' | b'L' | b'Q' => {
                self.flush_before_command();
                self.parse_expression()?;
            }
            b'K' | b'S' => {
                self.flush_before_command();
                self.take(1, "单字节参数")?;
            }
            b'J' | b'T' | b'V' | b'Y' | b'Z' => {
                self.flush_before_command();
                let first = self.parse_expression()?;
                let second = self.parse_expression()?;
                if opcode == b'Y' && first.value == Some(8) {
                    self.menu_mode = second.value;
                }
            }
            b'O' => {
                self.flush_before_command();
                let mode = self.parse_expression()?.value.ok_or_else(|| {
                    format!("offset 0x{start:X}: O 的参数数量依赖运行时变量，拒绝猜测")
                })?;
                if mode == 0 {
                    self.parse_expression()?;
                }
            }
            b'N' => {
                self.flush_before_command();
                self.take(1, "N 子命令")?;
                self.parse_expressions(2)?;
            }
            b'W' => {
                self.flush_before_command();
                self.parse_expressions(3)?;
            }
            b'B' => {
                self.flush_before_command();
                self.take(1, "B 子命令")?;
                self.parse_expressions(6)?;
            }
            b'I' | b'E' => {
                self.flush_before_command();
                self.parse_expressions(6)?;
            }
            b'H' => {
                self.take(1, "H 子命令")?;
                self.parse_expression()?;
                let raw = &self.bytes[start..self.pos];
                self.ensure_pending(start, StringKind::DisplayText, None);
                self.pending
                    .as_mut()
                    .expect("pending exists")
                    .add_inline_command(start, raw, opcode);
            }
            b'X' => {
                self.take(1, "X 字符串变量编号")?;
                let raw = &self.bytes[start..self.pos];
                self.ensure_pending(start, StringKind::DisplayText, None);
                self.pending
                    .as_mut()
                    .expect("pending exists")
                    .add_inline_command(start, raw, opcode);
            }
            b'M' => {
                self.flush_before_command();
                if self.menu_mode.unwrap_or(0) != 0 {
                    self.parse_menu_string()?;
                }
            }
            b'$' => {
                if self.capture_mode {
                    self.flush_pending();
                    self.capture_mode = false;
                    self.capture_id = None;
                } else {
                    self.flush_pending();
                    let capture_id = self.take_absolute_u16("$ 捕获字符串编号")?;
                    self.capture_id = Some(capture_id);
                    self.capture_mode = true;
                }
            }
            b'A' => self.flush_pending(),
            b'U' => {
                self.flush_before_command();
                self.parse_expressions(2)?;
            }
            _ => {
                return Err(format!(
                    "offset 0x{start:X}: 未知 System 3 操作码 0x{opcode:02X}"
                ));
            }
        }
        Ok(())
    }

    fn parse_menu_string(&mut self) -> Result<()> {
        let start = self.pos;
        let mut text = String::new();
        let mut unmapped_glyphs = Vec::new();
        while self.pos < self.bytes.len() {
            if self.bytes[self.pos] == b':' {
                self.pos += 1;
                let pending = PendingString {
                    start,
                    end: self.pos - 1,
                    kind: StringKind::MenuText,
                    capture_id: None,
                    scr_msg: text.clone(),
                    parts: vec![MessagePart::Text {
                        offset: start,
                        size: self.pos - 1 - start,
                        raw_hex: hex(&self.bytes[start..self.pos - 1]),
                        text,
                        unmapped_glyphs,
                    }],
                };
                self.push_pending(pending);
                return Ok(());
            }
            let char_start = self.pos;
            let byte = self.bytes[self.pos];
            if byte == 0x20 || (0xA0..=0xDF).contains(&byte) {
                text.push_str(self.glyphs.decode_single(byte)?);
                self.pos += 1;
            } else if is_cp932_lead(byte) {
                let pair = self.take(2, "菜单 CP932 字符")?;
                match decode_cp932(pair) {
                    Ok(decoded) => text.push_str(&decoded),
                    Err(_) => {
                        let raw_hex = hex(pair);
                        let placeholder = format!("⟦{raw_hex}⟧");
                        text.push_str(&placeholder);
                        unmapped_glyphs.push(RawGlyph {
                            offset: char_start,
                            raw_hex,
                            placeholder,
                        });
                    }
                }
            } else {
                return Err(format!(
                    "菜单 offset 0x{char_start:X}: 非文本字节 0x{byte:02X}"
                ));
            }
        }
        Err(format!("offset 0x{start:X}: M 菜单字符串缺少 ':' 终止符"))
    }

    fn parse_expressions(&mut self, count: usize) -> Result<()> {
        for _ in 0..count {
            self.parse_expression()?;
        }
        Ok(())
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        let start = self.pos;
        let mut stack: Vec<Option<u16>> = Vec::new();
        loop {
            let token = *self
                .bytes
                .get(self.pos)
                .ok_or_else(|| format!("offset 0x{start:X}: 表达式未终止"))?;
            self.pos += 1;
            match token {
                0x7F => {
                    if stack.len() != 1 {
                        return Err(format!(
                            "offset 0x{start:X}: 表达式结束时栈深为 {}",
                            stack.len()
                        ));
                    }
                    return Ok(Expression {
                        value: stack.pop().expect("one value"),
                    });
                }
                0x80..=0xFF => {
                    if token & 0x40 != 0 {
                        self.take(1, "双字节变量编号")?;
                    }
                    stack.push(None);
                }
                0x00..=0x76 => {
                    let value = if token & 0x40 == 0 {
                        let low = self.take(1, "双字节常量")?[0];
                        (u16::from(token) << 8) | u16::from(low)
                    } else {
                        u16::from(token & 0x3F)
                    };
                    stack.push(Some(value));
                }
                0x77..=0x7E => {
                    let right = stack.pop().ok_or_else(|| {
                        format!("offset 0x{:X}: 运算符缺少右操作数", self.pos - 1)
                    })?;
                    let left = stack.pop().ok_or_else(|| {
                        format!("offset 0x{:X}: 运算符缺少左操作数", self.pos - 1)
                    })?;
                    let value = match (left, right) {
                        (Some(left), Some(right)) => Some(eval_operator(token, left, right)),
                        _ => None,
                    };
                    stack.push(value);
                }
            }
        }
    }

    fn skip_variable_selector(&mut self) -> Result<()> {
        let byte = self.take(1, "变量编号")?[0];
        if byte & 0x40 != 0 {
            self.take(1, "双字节变量编号")?;
        }
        Ok(())
    }

    fn take(&mut self, count: usize, what: &str) -> Result<&[u8]> {
        let end = self
            .pos
            .checked_add(count)
            .ok_or_else(|| format!("offset 0x{:X}: {what} 长度溢出", self.pos))?;
        let slice = self.bytes.get(self.pos..end).ok_or_else(|| {
            format!(
                "offset 0x{:X}: {what} 需要 {count} 字节但文件已结束",
                self.pos
            )
        })?;
        self.pos = end;
        Ok(slice)
    }

    fn take_absolute_u16(&mut self, what: &str) -> Result<u16> {
        let field_offset = self.pos;
        let raw = self.take(2, what)?;
        let value = u16::from_le_bytes([raw[0], raw[1]]);
        self.relocations.push(Relocation {
            field_offset,
            target_offset: usize::from(value),
            kind: RelocationKind::AbsoluteU16,
            base_offset: None,
        });
        Ok(value)
    }

    fn ensure_pending(&mut self, start: usize, kind: StringKind, capture_id: Option<u16>) {
        if self.pending.is_none() {
            self.pending = Some(PendingString::new(start, kind, capture_id));
        }
    }

    fn flush_before_command(&mut self) {
        if !self.capture_mode {
            self.flush_pending();
        }
    }

    fn flush_pending(&mut self) {
        if let Some(pending) = self.pending.take() {
            if pending.has_content() {
                self.push_pending(pending);
            }
        }
    }

    fn push_pending(&mut self, pending: PendingString) {
        self.strings.push(StringRecord {
            _index: self.strings.len() + 1,
            _offset: pending.start,
            _end: pending.end,
            _size: pending.end.saturating_sub(pending.start),
            _type: pending.kind,
            capture_id: pending.capture_id,
            scr_msg: pending.scr_msg,
            parts: pending.parts,
        });
    }

    fn remaining_is_zero(&self) -> bool {
        self.bytes[self.pos..].iter().all(|byte| *byte == 0)
    }
}

fn is_text_start(byte: u8) -> bool {
    byte == 0x20 || byte & 0x80 != 0
}

fn is_cp932_lead(byte: u8) -> bool {
    (0x80..=0x9F).contains(&byte) || byte >= 0xE0
}

fn decode_cp932(bytes: &[u8]) -> Result<String> {
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(bytes);
    if had_errors {
        return Err(format!("无效 CP932 字节 {}", hex(bytes)));
    }
    Ok(decoded.into_owned())
}

fn eval_operator(operator: u8, left: u16, right: u16) -> u16 {
    match operator {
        0x77 => left.wrapping_mul(right),
        0x78 => {
            if right == 0 {
                0xFFFF
            } else {
                left / right
            }
        }
        0x79 => left.wrapping_add(right),
        0x7A => left.wrapping_sub(right),
        0x7B => u16::from(left == right),
        0x7C => u16::from(left < right),
        0x7D => u16::from(left > right),
        0x7E => u16::from(left != right),
        _ => unreachable!("operator range checked"),
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut output, "{byte:02X}").expect("writing to String cannot fail");
    }
    output
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:X}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glyph_table() -> GlyphTable {
        let mut decoded = Vec::new();
        for index in 0..GLYPH_COUNT {
            decoded.push(char::from_u32(0xE000 + index as u32).unwrap().to_string());
        }
        GlyphTable {
            decoded,
            raw_sha256: "TEST".to_string(),
        }
    }

    #[test]
    fn parses_text_newline_and_advance() {
        let bytes = [0, 0, 0xA2, 0x82, 0xA0, b'R', 0x20, 0xA3, b'A', 0x1A];
        let analysis = analyze_script(&bytes, &glyph_table(), "sample").unwrap();
        assert_eq!(analysis.strings.len(), 1);
        assert_eq!(analysis.strings[0].scr_msg.chars().count(), 5);
        assert!(analysis.strings[0].scr_msg.contains('\n'));
        assert_eq!(analysis.terminator_offset, Some(9));
    }

    #[test]
    fn parses_expression_constants_and_operator() {
        let bytes = [0, 0, b'G', 0x40, 0x41, 0x79, 0x7F, 0x1A];
        let analysis = analyze_script(&bytes, &glyph_table(), "sample").unwrap();
        assert_eq!(analysis.command_counts.get("47"), Some(&1));
    }

    #[test]
    fn parses_capture_string() {
        let bytes = [0, 0, b'$', 0x34, 0x12, 0xA2, 0xA3, b'$', 0x1A];
        let analysis = analyze_script(&bytes, &glyph_table(), "sample").unwrap();
        assert_eq!(analysis.strings.len(), 1);
        assert_eq!(analysis.strings[0].capture_id, Some(0x1234));
    }

    #[test]
    fn rejects_unknown_opcode() {
        let error = analyze_script(&[0, 0, 0x01, 0x1A], &glyph_table(), "sample").unwrap_err();
        assert!(error.contains("未知 System 3 操作码"));
    }
}
