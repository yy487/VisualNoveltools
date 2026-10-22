use crate::Result;
use unicode_normalization::UnicodeNormalization;

const KANA: &str = "。「」、・ヲァィゥェォャュョッーアイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワン";

#[derive(Debug, Clone)]
pub struct Codec {
    pub macros: Vec<Vec<u8>>,
}

impl Codec {
    pub fn parse_macros(raw: &[u8]) -> Result<Self> {
        let mut macros = Vec::new();
        let mut pos = 0;
        for _ in 0..64 {
            let value = terminated(raw, pos)?.to_vec();
            if value[..value.len() - 1]
                .iter()
                .any(|b| (0x60..=0x9F).contains(b))
            {
                return Err("MSGMACRO 中存在引擎不支持的嵌套字典引用".into());
            }
            pos += value.len();
            macros.push(value);
        }
        let codec = Self { macros };
        for value in &codec.macros {
            codec.decode_plain(value, false)?;
            codec.decode_plain(value, true)?;
        }
        Ok(codec)
    }

    fn expanded(&self, raw: &[u8]) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        for &b in raw {
            match b {
                0xFF => return Ok(out),
                0x60..=0x9F => {
                    let value = self
                        .macros
                        .get(usize::from(b - 0x60))
                        .ok_or("字典编号越界")?;
                    out.extend_from_slice(&value[..value.len() - 1]);
                }
                _ => out.push(b),
            }
        }
        Err("文本缺少 FF 结束符".into())
    }

    pub fn decode(&self, raw: &[u8]) -> Result<String> {
        let mut expanded = self.expanded(raw)?;
        expanded.push(0xFF);
        Ok(self.decode_plain(&expanded, false)?.0)
    }

    fn decode_plain(&self, raw: &[u8], initial_mode: bool) -> Result<(String, bool)> {
        let mut out = String::new();
        let mut chunk = String::new();
        let mut mode = initial_mode;
        let flush = |chunk: &mut String, out: &mut String, mode: bool| {
            for mut c in chunk.nfkc() {
                if !mode && ('\u{30A1}'..='\u{30F6}').contains(&c) {
                    c = char::from_u32(c as u32 - 0x60).expect("kana codepoint");
                }
                out.push(c);
            }
            chunk.clear();
        };
        for &b in raw {
            match b {
                0xFF => {
                    flush(&mut chunk, &mut out, mode);
                    return Ok((out, mode));
                }
                1 => {
                    flush(&mut chunk, &mut out, mode);
                    mode = !mode;
                }
                0x20..=0x5F => chunk.push(char::from(b)),
                0xA1..=0xDF => chunk
                    .push(char::from_u32(0xFF61 + u32::from(b - 0xA1)).expect("halfwidth kana")),
                _ => return Err(format!("未知文本码 ${b:02X}，拒绝把它当作普通字符").into()),
            }
        }
        Err("文本缺少 FF 结束符".into())
    }

    /// Minimum-byte encoding using the existing, immutable 64-entry dictionary.
    pub fn encode(&self, message: &str) -> Result<Vec<u8>> {
        if message.nfc().collect::<String>() != message {
            return Err("译文须使用 NFC 组合假名，不能保留独立的组合浊点".into());
        }
        let positions = message
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(message.len()))
            .collect::<Vec<_>>();
        let n = positions.len() - 1;
        let mut literal = Vec::new();
        for c in message.chars() {
            literal.push(literal_character(c)?);
        }
        let mut words = Vec::new();
        for value in &self.macros {
            words.push([
                self.decode_plain(value, false)?,
                self.decode_plain(value, true)?,
            ]);
        }
        let mut best = vec![[None::<Vec<u8>>, None::<Vec<u8>>]; n + 1];
        best[n] = [Some(vec![0xFF]), Some(vec![0xFF])];
        for i in (0..n).rev() {
            for state in 0..2 {
                let (bytes, wanted) = &literal[i];
                let next = wanted.map_or(state, usize::from);
                let mut current = Vec::new();
                if next != state {
                    current.push(1);
                }
                current.extend_from_slice(bytes);
                current.extend_from_slice(best[i + 1][next].as_ref().expect("following state"));
                for (index, word) in words.iter().enumerate() {
                    let (text, end_mode) = &word[state];
                    if text.is_empty() || !message[positions[i]..].starts_with(text) {
                        continue;
                    }
                    let end = positions[i] + text.len();
                    let Ok(j) = positions.binary_search(&end) else {
                        continue;
                    };
                    let tail = best[j][usize::from(*end_mode)]
                        .as_ref()
                        .expect("dictionary successor");
                    if 1 + tail.len() < current.len() {
                        current = Vec::with_capacity(1 + tail.len());
                        current.push(0x60 + index as u8);
                        current.extend_from_slice(tail);
                    }
                }
                best[i][state] = Some(current);
            }
        }
        let encoded = best[0][0].take().expect("initial state");
        if self.decode(&encoded)? != message {
            return Err("译文编码往返不一致".into());
        }
        self.validate_cache(&encoded)?;
        Ok(encoded)
    }

    fn validate_cache(&self, raw: &[u8]) -> Result<()> {
        let mut slots = std::collections::BTreeSet::from([0x24]);
        let mut cells = 0;
        let mut mode = false;
        for b in self.expanded(raw)? {
            if b == 1 {
                mode = !mode;
                continue;
            }
            slots.insert(glyph_id(b, mode));
            if slots.len() > 67 {
                return Err("单页字形超过 67 个去重槽（含空格）".into());
            }
            if b != 0xDE && b != 0xDF {
                cells += 1;
            }
            if cells == 77 {
                cells = 0;
                slots = std::collections::BTreeSet::from([0x24]);
            }
        }
        Ok(())
    }
}

fn literal_character(c: char) -> Result<(Vec<u8>, Option<bool>)> {
    if c == ' ' || c.is_ascii_digit() || c.is_ascii_uppercase() || "!,.?".contains(c) {
        return Ok((vec![c as u8], None));
    }
    match c {
        '「' => return Ok((vec![0xA2], None)),
        '」' => return Ok((vec![0xA3], None)),
        'ー' => return Ok((vec![0xB0], None)),
        _ => {}
    }
    let mut bytes = Vec::new();
    let mut wanted = None;
    for mut ch in c.to_string().nfd() {
        match ch {
            '\u{3099}' => bytes.push(0xDE),
            '\u{309A}' => bytes.push(0xDF),
            _ => {
                let katakana = ('\u{30A1}'..='\u{30F6}').contains(&ch);
                if ('\u{3041}'..='\u{3096}').contains(&ch) {
                    ch = char::from_u32(ch as u32 + 0x60).expect("kana codepoint");
                } else if !katakana {
                    return Err(
                        format!("字符 {c:?} 没有原版字形／编码；16×16 中文引擎尚未接入").into(),
                    );
                }
                let index = KANA
                    .chars()
                    .position(|k| k == ch)
                    .ok_or_else(|| format!("不能编码假名 {c}"))?;
                let byte = 0xA1 + index as u8;
                if !(byte == 0xA6 || (0xAC..=0xDD).contains(&byte)) {
                    return Err(format!("原版 glyph 映射不支持 {c}").into());
                }
                wanted = Some(katakana);
                bytes.push(byte);
            }
        }
    }
    if bytes.is_empty() {
        return Err(format!("空编码: {c:?}").into());
    }
    Ok((bytes, wanted))
}

pub fn glyph_id(b: u8, mode: bool) -> u8 {
    match b {
        0xDF => 0x2D,
        0xDE => 0x2C,
        0xB1..=0xFE => b - if mode { 0x82 } else { 0x50 },
        0xB0 => 0x2E,
        0xAC..=0xAF => b - if mode { 0x4F } else { 0x1D },
        0xA6 => {
            if mode {
                0x5C
            } else {
                0x8E
            }
        }
        0xA3 => 0x2B,
        0xA2 => 0x2A,
        0x3F => 0x29,
        0x41..=0xAB => b - 0x37,
        0x30..=0x40 => b - 0x30,
        0x2E => 0x26,
        0x2C => 0x25,
        0x21 => 0x28,
        _ => 0x24,
    }
}

pub fn terminated(raw: &[u8], start: usize) -> Result<&[u8]> {
    let suffix = raw.get(start..).ok_or("文本指针越界")?;
    let end = suffix
        .iter()
        .position(|b| *b == 0xFF)
        .ok_or("文本缺少 FF 结束符")?;
    Ok(&suffix[..=end])
}
