//! MZ wrappers and the fixed native-string spans established by disassembly.
use crate::{Game, Result, mes, word};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vn_text::Entry;

#[derive(Clone)]
pub struct Module {
    pub runtime: Vec<u8>,
    stub: Option<Vec<u8>>,
    header: Vec<u8>,
}

fn put(b: &mut [u8], p: usize, value: usize) -> Result<()> {
    let value = u16::try_from(value).map_err(|_| "MZ 字段超过 16 位")?;
    b.get_mut(p..p + 2)
        .ok_or("MZ 头截断")?
        .copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn size_header(header: &mut [u8], length: usize) -> Result<()> {
    put(header, 2, length % 512)?;
    put(header, 4, length.div_ceil(512))
}

fn expand(body: &[u8], pack: usize) -> Result<Vec<u8>> {
    let mut out = vec![0; usize::from(word(body, pack + 12)?) * 16];
    let (mut s, mut d) = (pack, out.len());
    while s != 0 && body[s - 1] == 255 {
        s -= 1;
    }
    for _ in 0..65536 {
        if s < 3 {
            return Err("EXEPACK 命令截断".into());
        }
        let command = body[s - 1];
        let n = usize::from(word(body, s - 3)?);
        s -= 3;
        let start = d.checked_sub(n).ok_or("EXEPACK 输出越界")?;
        match command & 0xfe {
            0xb0 => {
                s = s.checked_sub(1).ok_or("EXEPACK 缺少填充值")?;
                out[start..d].fill(body[s]);
            }
            0xb2 => {
                let p = s.checked_sub(n).ok_or("EXEPACK 数据截断")?;
                out[start..d].copy_from_slice(&body[p..s]);
                s = p;
            }
            _ => return Err("未知 EXEPACK 命令".into()),
        }
        d = start;
        if command & 1 != 0 {
            if d != s {
                return Err("EXEPACK 原样前缀不匹配".into());
            }
            out[..d].copy_from_slice(&body[..s]);
            return Ok(out);
        }
    }
    Err("EXEPACK 命令过多".into())
}

pub fn module(source: &[u8]) -> Result<Module> {
    if source.get(..2) != Some(b"MZ") {
        return Err("原生程序不是 MZ".into());
    }
    let head = usize::from(word(source, 8)?) * 16;
    if !(28..=65536).contains(&head) {
        return Err("MZ 头长度非法".into());
    }
    let mut header = source.get(..head).ok_or("MZ 头截断")?.to_vec();
    let mut body = source[head..].to_vec();
    let (mut ip, mut cs) = (
        usize::from(word(source, 20)?),
        usize::from(word(source, 22)?),
    );
    let entry = cs * 16 + ip;
    if body.get(entry..entry + 12)
        == Some(&[
            0xb8, 0x0a, 0xe9, 0xbb, 0xab, 0x37, 0xbf, 0, 1, 0xb9, 0xe3, 0x32,
        ])
    {
        let mut state = 0xe90au32;
        for value in body.get_mut(..0x32e3).ok_or("ADVH XOR 数据截断")? {
            let product = state * 0x37ab;
            state = (product + 0x31b1) & 65535;
            *value ^= ((product >> 16) & 255) as u8;
        }
        body.get_mut(0x2ad7..0x2adb)
            .ok_or("ADVH 启动补丁越界")?
            .copy_from_slice(&[0x80, 1, 0xdb, 0x40]);
        ip = 0x10;
    }
    for layer in 0..=32 {
        let entry = cs
            .checked_mul(16)
            .and_then(|v| v.checked_add(ip))
            .ok_or("入口溢出")?;
        if body.get(entry..entry + 1) != Some(&[0xe8])
            || body.get(entry + 3..entry + 6) != Some(b"TC0")
        {
            break;
        }
        if layer == 32 {
            return Err("TC0 层数过多".into());
        }
        let mut state = word(&body, entry + 6)?;
        let count = usize::from(word(&body, entry + 8)?);
        let start = usize::from(word(&body, entry + 10)?)
            .checked_sub(256)
            .ok_or("TC0 地址位于 PSP")?;
        ip = usize::from(word(&body, entry + 12)?);
        cs = usize::from(word(&body, entry + 14)?);
        if start + count > entry {
            return Err("TC0 目标覆盖解密入口".into());
        }
        for value in body.get_mut(start..start + count).ok_or("TC0 数据截断")? {
            state = state.wrapping_mul(0x2711).wrapping_add(0x3619);
            *value ^= (state.rotate_left(2) >> 8) as u8;
        }
    }
    let pack = cs * 16;
    if body.get(pack + 14..pack + 16) != Some(b"RB") {
        return Ok(Module {
            runtime: source.to_vec(),
            stub: None,
            header,
        });
    }
    let expanded = expand(&body, pack)?;
    let stub_length = usize::from(word(&body, pack + 6)?);
    let stub = body
        .get(pack..pack + stub_length)
        .ok_or("EXEPACK stub 截断")?
        .to_vec();
    let marker = [
        0x0e, 0x1f, 0x8b, 0x1e, 4, 0, 0xfc, 0x33, 0xd2, 0xad, 0x8b, 0xc8,
    ];
    let m = stub
        .windows(marker.len())
        .position(|b| b == marker)
        .ok_or("未知的 EXEPACK 重定位入口")?;
    if m < 3 || stub[m - 3] != 0xbe {
        return Err("EXEPACK 缺少重定位指针".into());
    }
    let mut p = usize::from(word(&stub, m - 2)?);
    let mut relocations = Vec::new();
    for group in 0..16 {
        let count = usize::from(word(&stub, p)?);
        p += 2;
        for _ in 0..count {
            relocations.push((word(&stub, p)?, group * 0x1000));
            p += 2;
        }
    }
    let runtime_head = (32 + relocations.len() * 4).div_ceil(16).max(32) * 16;
    let mut decoded_header = vec![0; runtime_head];
    decoded_header[..28].copy_from_slice(&header[..28]);
    put(&mut decoded_header, 6, relocations.len())?;
    put(&mut decoded_header, 8, runtime_head / 16)?;
    put(&mut decoded_header, 24, 32)?;
    put(&mut decoded_header, 18, 0)?;
    for (i, (offset, segment)) in relocations.into_iter().enumerate() {
        put(&mut decoded_header, 32 + i * 4, usize::from(offset))?;
        put(&mut decoded_header, 34 + i * 4, segment)?;
    }
    for (target, source_offset) in [(20, 0), (22, 2), (16, 8), (14, 10)] {
        put(
            &mut decoded_header,
            target,
            usize::from(word(&stub, source_offset)?),
        )?;
    }
    let memory = usize::from(word(&stub, 10)?) + usize::from(word(&stub, 8)?).div_ceil(16);
    put(
        &mut decoded_header,
        10,
        memory.saturating_sub(expanded.len() / 16),
    )?;
    size_header(&mut decoded_header, runtime_head + expanded.len())?;
    decoded_header.extend(expanded);
    // Repacking uses the ordinary EXEPACK stub; TC0/XOR wrappers are unnecessary
    // for edited text. Untouched programs always keep their exact original bytes.
    header.resize(512, 0);
    put(&mut header, 8, 32)?;
    Ok(Module {
        runtime: decoded_header,
        stub: Some(stub),
        header,
    })
}

fn rle(data: &[u8]) -> Vec<u8> {
    let mut chunks: Vec<(bool, Vec<u8>, usize)> = Vec::new();
    let mut raw = 0;
    let mut i = 0;
    let flush = |start: usize, end: usize, chunks: &mut Vec<(bool, Vec<u8>, usize)>| {
        for part in data[start..end].chunks(65535) {
            chunks.push((false, part.to_vec(), part.len()));
        }
    };
    while i < data.len() {
        let mut end = i + 1;
        while end < data.len() && data[end] == data[i] {
            end += 1;
        }
        if end - i >= 8 {
            flush(raw, i, &mut chunks);
            let mut left = end - i;
            while left != 0 {
                let n = left.min(65535);
                chunks.push((true, vec![data[i]], n));
                left -= n;
            }
            raw = end;
        }
        i = end;
    }
    flush(raw, data.len(), &mut chunks);
    let mut out = Vec::new();
    for (index, (fill, bytes, count)) in chunks.into_iter().enumerate() {
        out.extend(bytes);
        out.extend((count as u16).to_le_bytes());
        out.push((if fill { 0xb0 } else { 0xb2 }) | u8::from(index == 0));
    }
    out
}

impl Module {
    pub fn encode(&self, runtime: &[u8]) -> Result<Vec<u8>> {
        let Some(stub) = &self.stub else {
            return Ok(runtime.to_vec());
        };
        let head = usize::from(word(runtime, 8)?) * 16;
        let data = runtime.get(head..).ok_or("运行模块头越界")?;
        if data.len() != usize::from(word(stub, 12)?) * 16 {
            return Err("原生程序长度发生变化".into());
        }
        let mut packed = rle(data);
        packed.resize(packed.len().div_ceil(16) * 16, 255);
        let pack = packed.len();
        packed.extend(stub);
        let mut header = self.header.clone();
        put(&mut header, 6, 0)?;
        put(&mut header, 20, 16)?;
        put(&mut header, 22, pack / 16)?;
        let ss = data.len() / 16 + stub.len().div_ceil(16);
        put(&mut header, 14, ss)?;
        put(&mut header, 16, 128)?;
        let native_end = usize::from(word(stub, 10)?) + usize::from(word(stub, 8)?).div_ceil(16);
        let allocation = (ss + 8).max(native_end);
        put(
            &mut header,
            10,
            allocation.saturating_sub(packed.len().div_ceil(16)),
        )?;
        put(&mut header, 18, 0)?;
        size_header(&mut header, 512 + packed.len())?;
        header.extend(packed);
        Ok(header)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextMode {
    Zero,
    Dollar,
    Script,
    FixedDouble,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeRecord {
    pub offset: usize,
    pub length: usize,
    pub mode: TextMode,
    pub crlf: bool,
    pub entry: Entry,
    pub expected: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Primary {
    pub path: String,
    pub original: Vec<u8>,
    pub records: Vec<NativeRecord>,
}

#[derive(Deserialize)]
struct Catalog {
    game: Game,
    file: String,
    offset: usize,
    length: usize,
    mode: TextMode,
    expected: String,
}

fn unhex(text: &str) -> Result<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return Err("十六进制长度非法".into());
    }
    text.as_bytes()
        .chunks(2)
        .map(|b| {
            let s = std::str::from_utf8(b).map_err(|e| e.to_string())?;
            u8::from_str_radix(s, 16).map_err(|e| e.to_string())
        })
        .collect()
}

pub fn parse(game: Game, files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<Primary>> {
    let catalog: Vec<Catalog> =
        serde_json::from_str(include_str!("native_catalog.json")).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for (path, bytes) in files {
        let name = path.rsplit('/').next().unwrap_or(path);
        let matching = catalog
            .iter()
            .filter(|c| c.game == game && c.file.eq_ignore_ascii_case(name))
            .collect::<Vec<_>>();
        if matching.is_empty() {
            continue;
        }
        let runtime = if bytes.starts_with(b"MZ") {
            module(bytes)?.runtime
        } else {
            bytes.clone()
        };
        let mut records = Vec::new();
        for c in matching {
            let expected = unhex(&c.expected)?;
            let source = runtime
                .get(c.offset..c.offset + c.length)
                .ok_or("内嵌语句位置越界，不是研究版本")?;
            if source != expected {
                return Err(format!("{path}: 0x{:X} 原生文本与已确认版本不符", c.offset));
            }
            let text = mes::decode(source)?
                .replace("\r\n", "\n")
                .replace('\r', "\n");
            records.push(NativeRecord {
                offset: c.offset,
                length: c.length,
                mode: c.mode,
                crlf: source.windows(2).any(|b| b == b"\r\n"),
                entry: Entry {
                    name: None,
                    message: text,
                },
                expected,
            });
        }
        records.sort_by_key(|r| r.offset);
        for pair in records.windows(2) {
            if pair[0].offset + pair[0].length > pair[1].offset {
                return Err("原生文本目录重叠".into());
            }
        }
        out.push(Primary {
            path: path.clone(),
            original: bytes.clone(),
            records,
        });
    }
    Ok(out)
}

fn formats(s: &str) -> Vec<String> {
    let b = s.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i + 1 < b.len() {
        if b[i] == b'%' && matches!(b[i + 1], b'c' | b's' | b'd' | b'u' | b'%') {
            out.push(s[i..i + 2].into());
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}

pub fn changed_texts(primary: &Primary, entries: &[Entry]) -> Result<Vec<String>> {
    vn_text::validate_shape(
        &primary
            .records
            .iter()
            .map(|r| r.entry.clone())
            .collect::<Vec<_>>(),
        entries,
    )
    .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for (r, e) in primary.records.iter().zip(entries) {
        if r.entry == *e {
            continue;
        }
        if e.message.chars().any(|c| c.is_control() && c != '\n') {
            return Err("内嵌语句不能含 NUL/CR/未定义控制符".into());
        }
        if r.mode == TextMode::Dollar && e.message.contains('$') {
            return Err("DOS 字串不能插入半角 $".into());
        }
        if formats(&r.entry.message) != formats(&e.message) {
            return Err("不能改变原生文本的 %c/%s 等格式占位符".into());
        }
        out.push(e.message.replace('\n', ""));
    }
    Ok(out)
}

pub fn rebuild(
    primary: &Primary,
    entries: &[Entry],
    encode: &impl Fn(&str, bool) -> Result<Vec<u8>>,
) -> Result<Vec<u8>> {
    changed_texts(primary, entries)?;
    if primary
        .records
        .iter()
        .zip(entries)
        .all(|(r, e)| r.entry == *e)
    {
        return Ok(primary.original.clone());
    }
    let wrapped = primary.original.starts_with(b"MZ");
    let module = if wrapped {
        Some(module(&primary.original)?)
    } else {
        None
    };
    let mut out = module
        .as_ref()
        .map_or_else(|| primary.original.clone(), |m| m.runtime.clone());
    for (r, e) in primary.records.iter().zip(entries) {
        if r.entry == *e {
            continue;
        }
        let script = r.mode == TextMode::Script;
        let mut bytes = Vec::new();
        for (i, line) in e.message.split('\n').enumerate() {
            if i != 0 {
                if script {
                    bytes.push(0xa5);
                } else if r.crlf {
                    bytes.extend(b"\r\n");
                } else {
                    bytes.push(b'\n');
                }
            }
            bytes.extend(encode(&mes::normalize(line, !script)?, !script)?);
        }
        if bytes.len() > r.length {
            return Err(format!(
                "{}: 内嵌槽 0x{:X} 最多 {} 字节，译文 {} 字节",
                primary.path,
                r.offset,
                r.length,
                bytes.len()
            ));
        }
        if r.mode == TextMode::FixedDouble
            && (!bytes.len().is_multiple_of(2)
                || bytes
                    .chunks_exact(2)
                    .any(|p| !mes::is_lead(Game::Lipstick3, p[0])))
        {
            return Err("固定双字节显示槽不能使用单字节 ASCII".into());
        }
        let fill = match r.mode {
            TextMode::Zero => 0,
            TextMode::Dollar => b' ',
            TextMode::Script => 0xd6,
            TextMode::FixedDouble => 0,
        };
        if r.mode == TextMode::FixedDouble {
            while bytes.len() < r.length {
                bytes.extend([0x81, 0x40]);
            }
        } else {
            bytes.resize(r.length, fill);
        }
        out[r.offset..r.offset + r.length].copy_from_slice(&bytes);
    }
    if let Some(m) = module {
        m.encode(&out)
    } else {
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backward_exepack_commands_preserve_raw_prefix_and_long_zero_runs() {
        let mut b = (0..=255).cycle().take(1300).collect::<Vec<u8>>();
        b.extend(vec![0; 90000]);
        b.extend([9; 33]);
        b.extend([1, 2, 3, 4]);
        b.resize(b.len().div_ceil(16) * 16, 0);
        let mut encoded = rle(&b);
        encoded.resize(encoded.len().div_ceil(16) * 16, 255);
        let pack = encoded.len();
        encoded.extend([0; 16]);
        encoded[pack + 12..pack + 14].copy_from_slice(&((b.len() / 16) as u16).to_le_bytes());
        assert_eq!(expand(&encoded, pack).unwrap(), b);
    }
}
