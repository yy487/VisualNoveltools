use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result, bail, ensure};
use encoding_rs::SHIFT_JIS;

fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
    let bytes = data
        .get(offset..offset + 4)
        .with_context(|| format!("读取 u32 越界：0x{offset:X}"))?;
    Ok(u32::from_le_bytes(bytes.try_into().expect("4-byte slice")))
}

fn write_u32(data: &mut [u8], offset: usize, value: u32) -> Result<()> {
    let dst = data
        .get_mut(offset..offset + 4)
        .with_context(|| format!("写入 u32 越界：0x{offset:X}"))?;
    dst.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscript {
    pub index: usize,
    pub name: String,
    pub length_offset: usize,
    pub start: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKind {
    Message,
    Choice,
}

impl TextKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Choice => "choice",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextLocation {
    pub subscript_index: usize,
    pub subscript_name: String,
    pub instruction_offset: usize,
    pub subscript_offset: usize,
    pub text_offset: usize,
    pub byte_length: usize,
    pub opcode: u8,
    pub kind: TextKind,
    pub choice_index: Option<usize>,
    pub raw: Vec<u8>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpRef {
    pub subscript_index: usize,
    pub field_offset: usize,
    pub target_relative: u32,
}

#[derive(Debug, Clone)]
pub struct ScriptAnalysis {
    pub subscripts: Vec<Subscript>,
    pub texts: Vec<TextLocation>,
    pub jumps: Vec<JumpRef>,
    pub instruction_count: usize,
}

pub fn is_cp932_double_byte_stream(raw: &[u8]) -> bool {
    if !raw.len().is_multiple_of(2) {
        return false;
    }
    raw.chunks_exact(2).all(|pair| {
        let lead = pair[0];
        let trail = pair[1];
        let lead_ok = (0x81..=0x9F).contains(&lead) || (0xE0..=0xFC).contains(&lead);
        let trail_ok = (0x40..=0x7E).contains(&trail) || (0x80..=0xFC).contains(&trail);
        lead_ok && trail_ok
    })
}

pub fn decode_cp932(raw: &[u8], label: &str) -> Result<String> {
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(raw);
    ensure!(!had_errors, "{label}: 不是合法 CP932");
    Ok(decoded.into_owned())
}

pub fn encode_cp932_double_byte(text: &str, label: &str) -> Result<Vec<u8>> {
    ensure!(!text.contains('\0'), "{label}: message 不能包含 NUL");
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        let mut unsupported = Vec::new();
        for character in text.chars() {
            let (_, _, character_error) = SHIFT_JIS.encode(&character.to_string());
            if character_error && !unsupported.contains(&character) {
                unsupported.push(character);
            }
        }
        let display = unsupported
            .iter()
            .map(|character| format!("{character}(U+{:04X})", *character as u32))
            .collect::<Vec<_>>()
            .join(", ");
        bail!("{label}: 存在 CP932 不可编码字符：{display}");
    }
    let encoded = encoded.into_owned();
    ensure!(
        is_cp932_double_byte_stream(&encoded),
        "{label}: FF 正文必须全部由 CP932 双字节字符组成；请改用全角字母、数字和标点"
    );
    Ok(encoded)
}

pub fn parse_tpc32(data: &[u8]) -> Result<Vec<Subscript>> {
    ensure!(data.get(..5) == Some(b"TPC32"), "数据不是 TPC32");
    let code_count = read_u32(data, 0x18)? as usize;
    let metadata = 0x20usize
        .checked_add(code_count.checked_mul(4).context("TPC32 代码表溢出")?)
        .context("TPC32 元数据偏移溢出")?;
    ensure!(metadata + 0x30 <= data.len(), "TPC32 元数据越界");
    let e_count = read_u32(data, metadata + 0x08)? as usize;
    let variable_group_count = read_u32(data, metadata + 0x0C)? as usize;

    let mut cursor = metadata + 0x30;
    let mut sub_counts = Vec::with_capacity(variable_group_count);
    for _ in 0..variable_group_count {
        ensure!(cursor + 2 <= data.len(), "TPC32 变量组截断");
        let sub_count = data[cursor + 1] as usize;
        cursor += 2;
        sub_counts.push(sub_count);
        for _ in 0..sub_count {
            ensure!(cursor + 2 <= data.len(), "TPC32 变量子组截断");
            let inner_count = data[cursor + 1] as usize;
            cursor += 2;
            for _ in 0..inner_count {
                let count = *data.get(cursor).context("TPC32 变量内层截断")? as usize;
                cursor = cursor
                    .checked_add(1 + count * 3)
                    .context("TPC32 变量内层范围溢出")?;
                ensure!(cursor <= data.len(), "TPC32 变量内层数据截断");
            }
        }
    }

    for sub_count in sub_counts {
        let length = *data.get(cursor).context("TPC32 变量名截断")? as usize;
        cursor = cursor
            .checked_add(1 + length)
            .context("TPC32 变量名范围溢出")?;
        ensure!(cursor <= data.len(), "TPC32 变量名越界");
        for _ in 0..sub_count {
            let length = *data.get(cursor).context("TPC32 变量子名截断")? as usize;
            cursor = cursor
                .checked_add(1 + length)
                .context("TPC32 变量子名范围溢出")?;
            ensure!(cursor <= data.len(), "TPC32 变量子名越界");
        }
    }

    let mut names = vec!["A".to_owned(), "B".to_owned(), "C".to_owned()];
    names.extend((0..e_count).map(|index| format!("e{index}")));
    let mut subscripts = Vec::with_capacity(names.len());
    for (index, name) in names.into_iter().enumerate() {
        let length_offset = cursor;
        let length = read_u32(data, cursor)? as usize;
        let start = cursor + 4;
        cursor = start
            .checked_add(length)
            .with_context(|| format!("子脚本 {name} 范围溢出"))?;
        ensure!(cursor <= data.len(), "子脚本 {name} 越过 TPC32 末尾");
        subscripts.push(Subscript {
            index,
            name,
            length_offset,
            start,
            length,
        });
    }
    ensure!(
        cursor == data.len(),
        "TPC32 存在尾随数据：子脚本结束 0x{cursor:X}，文件结束 0x{:X}",
        data.len()
    );
    Ok(subscripts)
}

struct Decoder<'a> {
    file_name: &'a str,
    data: &'a [u8],
    subscript: &'a Subscript,
    end: usize,
    cursor: usize,
    texts: Vec<TextLocation>,
    jumps: Vec<JumpRef>,
    instruction_starts: HashSet<usize>,
    instruction_count: usize,
}

impl<'a> Decoder<'a> {
    fn new(file_name: &'a str, data: &'a [u8], subscript: &'a Subscript) -> Self {
        Self {
            file_name,
            data,
            subscript,
            end: subscript.start + subscript.length,
            cursor: subscript.start,
            texts: Vec::new(),
            jumps: Vec::new(),
            instruction_starts: HashSet::new(),
            instruction_count: 0,
        }
    }

    fn location(&self, offset: usize) -> String {
        format!(
            "{}:{}+0x{:X} (TPC 0x{offset:X})",
            self.file_name,
            self.subscript.name,
            offset.saturating_sub(self.subscript.start)
        )
    }

    fn need(&self, count: usize) -> Result<()> {
        ensure!(
            self.cursor
                .checked_add(count)
                .is_some_and(|end| end <= self.end),
            "{}: 需要 {count} 字节，但子脚本已结束",
            self.location(self.cursor)
        );
        Ok(())
    }

    fn byte(&mut self) -> Result<u8> {
        self.need(1)?;
        let value = self.data[self.cursor];
        self.cursor += 1;
        Ok(value)
    }

    fn bytes(&mut self, count: usize) -> Result<&'a [u8]> {
        self.need(count)?;
        let start = self.cursor;
        self.cursor += count;
        Ok(&self.data[start..self.cursor])
    }

    fn varint(&mut self) -> Result<()> {
        self.need(1)?;
        let width_code = (self.data[self.cursor] >> 4) & 7;
        let width = usize::from(width_code.max(1));
        self.bytes(width)?;
        Ok(())
    }

    fn varints(&mut self, count: usize) -> Result<()> {
        for _ in 0..count {
            self.varint()?;
        }
        Ok(())
    }

    fn cstring(&mut self) -> Result<(usize, &'a [u8])> {
        let start = self.cursor;
        let relative_end = self.data[start..self.end]
            .iter()
            .position(|&byte| byte == 0)
            .with_context(|| format!("{}: 字符串缺少 NUL", self.location(start)))?;
        let end = start + relative_end;
        self.cursor = end + 1;
        Ok((start, &self.data[start..end]))
    }

    fn simple_string(&mut self) -> Result<()> {
        self.need(1)?;
        if self.data[self.cursor] == 0x40 {
            self.cursor += 1;
            self.varint()
        } else {
            self.cstring().map(|_| ())
        }
    }

    fn condition(&mut self) -> Result<()> {
        let start = self.cursor;
        ensure!(
            self.byte()? == 0x28,
            "{}: 条件表达式不以 '(' 开始",
            self.location(start)
        );
        let mut depth = 1usize;
        while depth != 0 {
            let token = self.byte()?;
            match token {
                0x28 => depth += 1,
                0x29 => depth -= 1,
                0x26 | 0x27 => {}
                0x36..=0x55 => self.varints(2)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn jump_u32(&mut self) -> Result<()> {
        let field_offset = self.cursor;
        let target_relative = read_u32(self.data, field_offset)?;
        self.cursor += 4;
        self.jumps.push(JumpRef {
            subscript_index: self.subscript.index,
            field_offset,
            target_relative,
        });
        Ok(())
    }

    fn add_text(
        &mut self,
        instruction_offset: usize,
        opcode: u8,
        kind: TextKind,
        choice_index: Option<usize>,
    ) -> Result<()> {
        let (text_offset, raw) = self.cstring()?;
        if opcode == 0xFF {
            ensure!(
                is_cp932_double_byte_stream(raw),
                "{}: FF 字符串不是纯 CP932 双字节流",
                self.location(text_offset)
            );
        }
        let text = decode_cp932(raw, &self.location(text_offset))?;
        self.texts.push(TextLocation {
            subscript_index: self.subscript.index,
            subscript_name: self.subscript.name.clone(),
            instruction_offset,
            subscript_offset: instruction_offset - self.subscript.start,
            text_offset,
            byte_length: raw.len(),
            opcode,
            kind,
            choice_index,
            raw: raw.to_vec(),
            text,
        });
        Ok(())
    }

    fn composite_string(&mut self) -> Result<()> {
        loop {
            match self.byte()? {
                0 => return Ok(()),
                0xFF | 0xFE => {
                    self.cstring()?;
                }
                0xFD => self.varint()?,
                marker => {
                    bail!(
                        "{}: 动态拼接串出现未知标记 0x{marker:02X}",
                        self.location(self.cursor - 1)
                    )
                }
            }
        }
    }

    fn embedded_format(&mut self) -> Result<()> {
        match self.byte()? {
            1 | 3 => self.varint(),
            2 => self.varints(2),
            _ => Ok(()),
        }
    }

    fn op_0b(&mut self) -> Result<()> {
        let subtype = self.byte()?;
        match subtype {
            0x22 => {
                let count = self.byte()? as usize;
                self.simple_string()?;
                self.varint()?;
                for _ in 0..count {
                    let item_type = self.byte()?;
                    self.simple_string()?;
                    match item_type {
                        2 => self.varint()?,
                        3 => self.varints(6)?,
                        4 => self.varints(7)?,
                        _ => {}
                    }
                }
            }
            0x30 => {}
            1 | 3 | 5 | 0x10 => {
                self.simple_string()?;
                self.varint()?;
            }
            6 => {
                self.simple_string()?;
                self.varints(15)?;
            }
            _ => bail!(
                "{}: 未覆盖 0x0B 子命令 0x{subtype:02X}",
                self.location(self.cursor - 1)
            ),
        }
        Ok(())
    }

    fn op_0e(&mut self) -> Result<()> {
        let subtype = self.byte()?;
        match subtype {
            1 | 0x30 | 0x32 => self.simple_string()?,
            5 => {
                self.simple_string()?;
                self.varint()?;
            }
            0x10 | 0x21 => self.varint()?,
            0x11 | 0x36 => {}
            _ => bail!(
                "{}: 未覆盖 0x0E 子命令 0x{subtype:02X}",
                self.location(self.cursor - 1)
            ),
        }
        Ok(())
    }

    fn op_19(&mut self) -> Result<()> {
        let subtype = self.byte()?;
        match subtype {
            1 => self.varint()?,
            2 => self.varints(2)?,
            0x10 | 0x11 | 0x13 => {}
            _ => bail!(
                "{}: 未覆盖 0x19 子命令 0x{subtype:02X}",
                self.location(self.cursor - 1)
            ),
        }
        Ok(())
    }

    fn op_58(&mut self, instruction_offset: usize) -> Result<()> {
        let subtype = self.byte()?;
        match subtype {
            4 | 5 => {
                self.varint()?;
                return Ok(());
            }
            1..=3 => {}
            _ => {
                bail!(
                    "{}: 未覆盖 0x58 子命令 0x{subtype:02X}",
                    self.location(self.cursor - 1)
                )
            }
        }
        self.varint()?;
        if subtype == 3 {
            self.varint()?;
        }
        if self.cursor < self.end && self.data[self.cursor] == 0x22 {
            self.cursor += 1;
        }
        if self.cursor < self.end && self.data[self.cursor] == 0 {
            self.cursor += 1;
        }

        let mut choice_index = 0usize;
        loop {
            self.need(1)?;
            if self.data[self.cursor] == 0x23 {
                self.cursor += 1;
                return Ok(());
            }
            let text_count_before = self.texts.len();
            loop {
                let marker_offset = self.cursor;
                match self.byte()? {
                    0 => break,
                    0x28 => {
                        self.cursor -= 1;
                        self.condition()?;
                    }
                    0xFF => self.add_text(
                        instruction_offset,
                        0xFF,
                        TextKind::Choice,
                        Some(choice_index),
                    )?,
                    0xFE => {
                        self.cstring()?;
                    }
                    0x10 => self.embedded_format()?,
                    marker => {
                        bail!(
                            "{}: 选项块出现未知标记 0x{marker:02X}",
                            self.location(marker_offset)
                        )
                    }
                }
            }
            let text_count = self.texts.len() - text_count_before;
            ensure!(
                text_count <= 1,
                "{}: 选项 {choice_index} 含有 {text_count} 个 FF 片段，需启用 message_parts 后再处理",
                self.location(instruction_offset)
            );
            choice_index += 1;
        }
    }

    fn decode(mut self) -> Result<(Vec<TextLocation>, Vec<JumpRef>, usize)> {
        while self.cursor < self.end {
            let instruction_offset = self.cursor;
            self.instruction_starts.insert(instruction_offset);
            let opcode = self.byte()?;
            if opcode == 0 {
                ensure!(
                    self.data[self.cursor..self.end]
                        .iter()
                        .all(|&byte| byte == 0),
                    "{}: 脚本终止符后存在非零数据",
                    self.location(instruction_offset)
                );
                self.cursor = self.end;
                break;
            }
            self.instruction_count += 1;
            match opcode {
                0x01 | 0x03 => {}
                0x02 | 0x04 | 0x20 => {
                    self.byte()?;
                }
                0x0B => self.op_0b()?,
                0x0E => self.op_0e()?,
                0x13 => match self.byte()? {
                    1 | 0x10 => self.varint()?,
                    subtype => {
                        bail!(
                            "{}: 未覆盖 0x13 子命令 0x{subtype:02X}",
                            self.location(self.cursor - 1)
                        )
                    }
                },
                0x15 => {
                    self.condition()?;
                    self.jump_u32()?;
                }
                0x16 => {
                    let subtype = self.byte()?;
                    ensure!(
                        subtype == 1,
                        "{}: 未覆盖 0x16 子命令 0x{subtype:02X}",
                        self.location(self.cursor - 1)
                    );
                    self.varint()?;
                }
                0x19 => self.op_19()?,
                0x1B | 0x1C => self.jump_u32()?,
                0x1D | 0x1E => {
                    let count = self.byte()? as usize;
                    self.varint()?;
                    for _ in 0..count {
                        self.jump_u32()?;
                    }
                }
                0x37 | 0x3B | 0x3C | 0x3D | 0x49 => self.varints(2)?,
                0x58 => self.op_58(instruction_offset)?,
                0x5C => {
                    self.byte()?;
                    self.varints(3)?;
                }
                0x60 => match self.byte()? {
                    2 => self.varint()?,
                    4 => self.composite_string()?,
                    0x20 => {}
                    subtype => {
                        bail!(
                            "{}: 未覆盖 0x60 子命令 0x{subtype:02X}",
                            self.location(self.cursor - 1)
                        )
                    }
                },
                0x64 => {
                    let subtype = self.byte()?;
                    ensure!(
                        subtype == 0x20,
                        "{}: 未覆盖 0x64 子命令 0x{subtype:02X}",
                        self.location(self.cursor - 1)
                    );
                    self.varints(5)?;
                }
                0x67 => {
                    let subtype = self.byte()?;
                    ensure!(
                        subtype == 1,
                        "{}: 未覆盖 0x67 子命令 0x{subtype:02X}",
                        self.location(self.cursor - 1)
                    );
                    self.varints(8)?;
                }
                0x6D => match self.byte()? {
                    2 => self.varints(3)?,
                    3 | 0x20 | 0x21 => {}
                    subtype => {
                        bail!(
                            "{}: 未覆盖 0x6D 子命令 0x{subtype:02X}",
                            self.location(self.cursor - 1)
                        )
                    }
                },
                0x72 => {
                    self.byte()?;
                    self.varints(2)?;
                }
                0x73 => {
                    let subtype = self.byte()?;
                    self.varint()?;
                    match subtype {
                        2 | 6 => self.varint()?,
                        0x1D => {}
                        _ => {
                            bail!(
                                "{}: 未覆盖 0x73 子命令 0x{subtype:02X}",
                                self.location(self.cursor - 1)
                            )
                        }
                    }
                }
                0x74 => match self.byte()? {
                    2 => self.varint()?,
                    4 => self.varints(2)?,
                    subtype => {
                        bail!(
                            "{}: 未覆盖 0x74 子命令 0x{subtype:02X}",
                            self.location(self.cursor - 1)
                        )
                    }
                },
                0x76 => {
                    let subtype = self.byte()?;
                    ensure!(
                        subtype == 1,
                        "{}: 未覆盖 0x76 子命令 0x{subtype:02X}",
                        self.location(self.cursor - 1)
                    );
                    self.varint()?;
                }
                0xFE => {
                    self.cstring()?;
                }
                0xFF => {
                    self.add_text(instruction_offset, 0xFF, TextKind::Message, None)?;
                }
                _ => {
                    bail!(
                        "{}: 未覆盖主操作码 0x{opcode:02X}",
                        self.location(instruction_offset)
                    )
                }
            }
        }
        ensure!(
            self.cursor == self.end,
            "{}: 解码结束位置不匹配",
            self.location(self.cursor)
        );

        for jump in &self.jumps {
            let target = self
                .subscript
                .start
                .checked_add(jump.target_relative as usize)
                .context("跳转目标溢出")?;
            ensure!(
                target == self.end || self.instruction_starts.contains(&target),
                "{}: 跳转目标 +0x{:X} 不是指令边界",
                self.location(jump.field_offset),
                jump.target_relative
            );
        }
        Ok((self.texts, self.jumps, self.instruction_count))
    }
}

pub fn analyze_tpc32(file_name: &str, data: &[u8]) -> Result<ScriptAnalysis> {
    let subscripts = parse_tpc32(data).with_context(|| format!("{file_name}: TPC32 解析失败"))?;
    let mut texts = Vec::new();
    let mut jumps = Vec::new();
    let mut instruction_count = 0usize;
    for subscript in &subscripts {
        let decoder = Decoder::new(file_name, data, subscript);
        let (mut sub_texts, mut sub_jumps, sub_instruction_count) = decoder.decode()?;
        texts.append(&mut sub_texts);
        jumps.append(&mut sub_jumps);
        instruction_count += sub_instruction_count;
    }
    Ok(ScriptAnalysis {
        subscripts,
        texts,
        jumps,
        instruction_count,
    })
}

#[derive(Debug, Clone)]
struct Edit {
    subscript_index: usize,
    offset: usize,
    old_length: usize,
    new_bytes: Vec<u8>,
}

pub fn patch_tpc32(
    original: &[u8],
    analysis: &ScriptAnalysis,
    replacements: &HashMap<usize, Vec<u8>>,
) -> Result<Vec<u8>> {
    let mut edits = Vec::with_capacity(replacements.len());
    for (&text_index, new_bytes) in replacements {
        let location = analysis
            .texts
            .get(text_index)
            .with_context(|| format!("文本索引 {text_index} 越界"))?;
        let old = original
            .get(location.text_offset..location.text_offset + location.byte_length)
            .context("原文位置越界")?;
        ensure!(
            old == location.raw,
            "{}:{} 0x{:X}: 原文字节验证失败",
            location.subscript_name,
            text_index,
            location.text_offset
        );
        edits.push(Edit {
            subscript_index: location.subscript_index,
            offset: location.text_offset,
            old_length: location.byte_length,
            new_bytes: new_bytes.clone(),
        });
    }
    if edits.is_empty() {
        return Ok(original.to_vec());
    }

    let mut out = original.to_vec();
    for subscript in &analysis.subscripts {
        let mut sub_edits = edits
            .iter()
            .filter(|edit| edit.subscript_index == subscript.index)
            .collect::<Vec<_>>();
        if sub_edits.is_empty() {
            continue;
        }
        sub_edits.sort_by_key(|edit| edit.offset);
        for pair in sub_edits.windows(2) {
            ensure!(
                pair[0].offset + pair[0].old_length <= pair[1].offset,
                "{}: 文本编辑范围重叠",
                subscript.name
            );
        }
        let total_delta = sub_edits.iter().try_fold(0i64, |sum, edit| {
            let new_length = i64::try_from(edit.new_bytes.len()).context("新文本过长")?;
            let old_length = i64::try_from(edit.old_length).context("原文本过长")?;
            sum.checked_add(new_length - old_length)
                .context("文本尺寸变化溢出")
        })?;
        let new_length = i64::try_from(subscript.length)
            .context("子脚本过长")?
            .checked_add(total_delta)
            .context("子脚本长度溢出")?;
        ensure!(new_length >= 0, "{}: 子脚本长度变为负数", subscript.name);
        write_u32(
            &mut out,
            subscript.length_offset,
            u32::try_from(new_length).context("子脚本长度超过 u32")?,
        )?;

        for jump in analysis
            .jumps
            .iter()
            .filter(|jump| jump.subscript_index == subscript.index)
        {
            let target_absolute = subscript
                .start
                .checked_add(jump.target_relative as usize)
                .context("跳转目标溢出")?;
            let delta = sub_edits.iter().try_fold(0i64, |sum, edit| {
                if edit.offset < target_absolute {
                    let new_length = i64::try_from(edit.new_bytes.len()).context("新文本过长")?;
                    let old_length = i64::try_from(edit.old_length).context("原文本过长")?;
                    sum.checked_add(new_length - old_length)
                        .context("跳转修正溢出")
                } else {
                    Ok(sum)
                }
            })?;
            let new_target = i64::from(jump.target_relative)
                .checked_add(delta)
                .context("跳转目标修正溢出")?;
            ensure!(new_target >= 0, "跳转目标变为负数");
            write_u32(
                &mut out,
                jump.field_offset,
                u32::try_from(new_target).context("跳转目标超过 u32")?,
            )?;
        }
    }

    edits.sort_by_key(|edit| std::cmp::Reverse(edit.offset));
    for edit in edits {
        out.splice(
            edit.offset..edit.offset + edit.old_length,
            edit.new_bytes.into_iter(),
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        JumpRef, ScriptAnalysis, Subscript, TextKind, TextLocation, encode_cp932_double_byte,
        is_cp932_double_byte_stream, patch_tpc32,
    };

    #[test]
    fn validates_ff_double_byte_rule() {
        assert!(is_cp932_double_byte_stream(&[0x82, 0x60, 0x81, 0x42]));
        assert!(!is_cp932_double_byte_stream(b"test"));
        assert!(encode_cp932_double_byte("Ａ。", "test").is_ok());
        assert!(encode_cp932_double_byte("A.", "test").is_err());
    }

    #[test]
    fn variable_length_edit_updates_length_and_jump() {
        let mut original = Vec::new();
        original.extend_from_slice(&11u32.to_le_bytes());
        original.extend_from_slice(&[0xFF, 0x82, 0x60, 0x00]);
        original.push(0x1C);
        original.extend_from_slice(&9u32.to_le_bytes());
        original.extend_from_slice(&[0x01, 0x00]);

        let analysis = ScriptAnalysis {
            subscripts: vec![Subscript {
                index: 0,
                name: "e0".to_owned(),
                length_offset: 0,
                start: 4,
                length: 11,
            }],
            texts: vec![TextLocation {
                subscript_index: 0,
                subscript_name: "e0".to_owned(),
                instruction_offset: 4,
                subscript_offset: 0,
                text_offset: 5,
                byte_length: 2,
                opcode: 0xFF,
                kind: TextKind::Message,
                choice_index: None,
                raw: vec![0x82, 0x60],
                text: "Ａ".to_owned(),
            }],
            jumps: vec![JumpRef {
                subscript_index: 0,
                field_offset: 9,
                target_relative: 9,
            }],
            instruction_count: 3,
        };
        let replacements = HashMap::from([(0usize, vec![0x82, 0x60, 0x82, 0x61])]);
        let patched = patch_tpc32(&original, &analysis, &replacements).expect("patch");
        assert_eq!(u32::from_le_bytes(patched[0..4].try_into().unwrap()), 13);
        assert_eq!(u32::from_le_bytes(patched[11..15].try_into().unwrap()), 11);
        assert_eq!(&patched[5..9], &[0x82, 0x60, 0x82, 0x61]);
        assert_eq!(patched[15], 0x01);
    }
}
