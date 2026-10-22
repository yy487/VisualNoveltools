//! Separate lexer contracts for ADVG, ADVH and Adv98V; flat translator records.
use crate::{Game, Result};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vn_text::Entry;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Arg {
    Number(u16),
    String(Vec<u8>),
    Variable(u8),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Text(String),
    ZeroText(String),
    Newline,
    State,
    Control(u16),
    Boundary(u16),
    Op(u8, Vec<Arg>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    pub span: Span,
    pub kind: Kind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encryption {
    pub start: usize,
    pub key: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    pub bytes: Vec<u8>,
    pub tokens: Vec<Token>,
    pub script_end: usize,
    pub encryption: Option<Encryption>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeakerOrigin {
    Literal {
        inner: Span,
    },
    Macro {
        call: Span,
        slot: u16,
        definition_file: String,
        definition_offset: usize,
        body: Vec<u8>,
        inner: Span,
    },
    ZeroString,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Speaker {
    pub name: String,
    pub origin: SpeakerOrigin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Script,
    ZeroString,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub span: Span,
    pub mode: Mode,
    pub speaker: Option<Speaker>,
    pub entry: Entry,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Primary {
    pub file: String,
    pub game: Game,
    pub original: Vec<u8>,
    pub player_pattern: Vec<u8>,
    pub records: Vec<Record>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Macro {
    pub file: String,
    pub offset: usize,
    pub body: Vec<u8>,
    pub name: Option<(String, Span)>,
    pub has_text: bool,
}
pub type Macros = BTreeMap<u16, Macro>;

#[derive(Clone, Debug)]
struct Definition {
    start: usize,
    end: usize,
    slot: u16,
    depth: usize,
    value: Macro,
}

pub fn decode(b: &[u8]) -> Result<String> {
    SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(b)
        .map(|s| s.into_owned())
        .ok_or_else(|| format!("无法按 CP932 解码 {b:02X?}"))
}

pub fn direct(s: &str) -> Option<Vec<u8>> {
    let (b, _, bad) = SHIFT_JIS.encode(s);
    (!bad).then(|| b.into_owned())
}

pub fn is_lead(game: Game, b: u8) -> bool {
    if game == Game::Lipstick3 {
        (0x81..=0x9f).contains(&b) || (0xe0..=0xfc).contains(&b)
    } else {
        (0x80..=0x9f).contains(&b) || b >= 0xe0
    }
}

pub fn controls(game: Game) -> &'static [u16] {
    match game {
        Game::Lipstick1 => &[
            0x8197, 0x8190, 0x816f, 0x8170, 0x81a8, 0x81a9, 0x81aa, 0x81ab,
        ],
        Game::Lipstick2 => &[0x8197, 0x8190, 0x816f, 0x8170, 0x81a8, 0x81ab],
        Game::Lipstick3 => &[0x8197, 0x8190, 0x816f, 0x8170],
    }
}

fn glyph_text(pair: [u8; 2]) -> String {
    if let Ok(text) = decode(&pair)
        && direct(&text).as_deref() == Some(pair.as_slice())
    {
        return text;
    }
    format!("{{glyph:{:02X}{:02X}}}", pair[0], pair[1])
}

fn raw_text(b: &[u8]) -> Result<String> {
    let mut out = String::new();
    let mut p = 0;
    while p < b.len() {
        if is_lead(Game::Lipstick3, b[p]) {
            let second = *b.get(p + 1).ok_or("原字串双字节字符截断")?;
            out.push_str(&glyph_text([b[p], second]));
            p += 2;
        } else {
            if b[p] == b'\r' {
                out.push('\n');
            } else if b[p] != b'\n' {
                out.push_str(&decode(&b[p..p + 1])?);
            }
            p += 1;
        }
    }
    Ok(out)
}

pub fn number(game: Game, b: &[u8], p: &mut usize) -> Result<Option<u16>> {
    if game == Game::Lipstick1 {
        while b.get(*p) == Some(&b' ') {
            *p += 1;
        }
        if b.get(*p) == Some(&b',') {
            *p += 1;
        }
        while b.get(*p) == Some(&b' ') {
            *p += 1;
        }
        let start = *p;
        let mut n = 0u16;
        while let Some(c @ b'0'..=b'9') = b.get(*p).copied() {
            n = n.wrapping_mul(10).wrapping_add(u16::from(c - b'0'));
            *p += 1;
        }
        return Ok((*p > start).then_some(n));
    }
    let Some(&tag) = b.get(*p) else {
        return Ok(None);
    };
    let (count, value) = match tag {
        0x23..=0x27 => (1, u16::from(tag - 0x23)),
        0x28 => (2, u16::from(*b.get(*p + 1).ok_or("28 数值截断")?)),
        0x29..=0x2c => {
            let tail = b.get(*p + 1..*p + 3).ok_or("长数值截断")?;
            if tail.iter().any(|x| *x > 0x7f) {
                return Err("长数值不是 7 位有效载荷".into());
            }
            (
                3,
                (u16::from(tag - 0x29) << 14) | (u16::from(tail[0]) << 7) | u16::from(tail[1]),
            )
        }
        _ => return Ok(None),
    };
    *p += count;
    Ok(Some(value))
}

pub fn encode_number(game: Game, n: u16) -> Vec<u8> {
    if game == Game::Lipstick1 {
        return n.to_string().into_bytes();
    }
    if n < 5 {
        vec![0x23 + n as u8]
    } else if n < 256 {
        vec![0x28, n as u8]
    } else {
        vec![
            0x29 + (n >> 14) as u8,
            ((n >> 7) & 127) as u8,
            (n & 127) as u8,
        ]
    }
}

fn numbers(game: Game, b: &[u8], p: &mut usize, count: usize) -> Result<Vec<Arg>> {
    let mut out = Vec::new();
    for _ in 0..count {
        if let Some(n) = number(game, b, p)? {
            out.push(Arg::Number(n));
        }
    }
    Ok(out)
}

fn quote(game: Game, b: &[u8], p: &mut usize) -> Result<Arg> {
    if game == Game::Lipstick1 {
        while b.get(*p) == Some(&b' ') {
            *p += 1;
        }
        if b.get(*p) == Some(&b',') {
            *p += 1;
        }
        while b.get(*p) == Some(&b' ') {
            *p += 1;
        }
    }
    if b.get(*p) != Some(&0x22) {
        return Err(format!("0x{:X}: 缺少引号参数", *p));
    }
    *p += 1;
    let start = *p;
    let end = b[start..]
        .iter()
        .position(|c| *c == 0x22)
        .ok_or("引号参数未闭合")?
        + start;
    *p = end + 1;
    Ok(Arg::String(b[start..end].to_vec()))
}

fn variable(b: &[u8], p: &mut usize) -> Result<Arg> {
    let value = *b.get(*p).ok_or("变量字母缺失")?;
    *p += 1;
    if !value.is_ascii_uppercase() {
        return Err(format!("变量名不是大写字母：{value:02X}"));
    }
    Ok(Arg::Variable(value))
}

fn first(args: &[Arg]) -> Option<u16> {
    args.iter().find_map(|a| {
        if let Arg::Number(n) = a {
            Some(*n)
        } else {
            None
        }
    })
}

fn operands(game: Game, code: u8, b: &[u8], p: &mut usize) -> Result<Vec<Arg>> {
    let mut out = match (game, code) {
        (_, 0xa6) => numbers(game, b, p, 4)?,
        (_, 0xa7 | 0xa9 | 0xb8 | 0xc3 | 0xc4 | 0xc5) => numbers(game, b, p, 2)?,
        (_, 0xa8) => numbers(game, b, p, 3)?,
        (_, 0xaa | 0xb5 | 0xb9 | 0xba | 0xbb | 0xbe) => numbers(game, b, p, 1)?,
        (Game::Lipstick3, 0xab | 0xac | 0xbf | 0xcc) => numbers(game, b, p, 2)?,
        (_, 0xab | 0xac | 0xbf) => numbers(game, b, p, 1)?,
        (_, 0xad | 0xae) => {
            let mut a = Vec::new();
            let max = if game == Game::Lipstick3 { 64 } else { 32 };
            for _ in 0..max {
                let Some(n) = number(game, b, p)? else {
                    break;
                };
                a.push(Arg::Number(n));
            }
            if !a.len().is_multiple_of(2) {
                return Err("菜单坐标缺少 y".into());
            }
            a
        }
        (_, 0xb0 | 0xb1) => {
            let mut a = vec![quote(game, b, p)?];
            if game == Game::Lipstick3 && code == 0xb0 {
                a.extend(numbers(game, b, p, 2)?);
            }
            a
        }
        (_, 0xb4) => vec![variable(b, p)?],
        (Game::Lipstick3, 0xb6) => {
            if b.get(*p) == Some(&0x22) {
                vec![quote(game, b, p)?]
            } else {
                let mut a = vec![variable(b, p)?];
                a.extend(numbers(game, b, p, 1)?);
                a
            }
        }
        (_, 0xb6) => {
            let mut a = Vec::new();
            loop {
                let Some(n) = number(game, b, p)? else {
                    break;
                };
                a.push(Arg::Number(n));
                a.extend(numbers(game, b, p, 4)?);
                if b.get(*p) != Some(&b',') {
                    break;
                }
                *p += 1;
            }
            if !a.len().is_multiple_of(5) {
                return Err("B6 热区参数不完整".into());
            }
            a
        }
        (_, 0xb7) => numbers(game, b, p, 3)?,
        (Game::Lipstick3, 0xc0) => {
            let mut a = Vec::new();
            if b.get(*p) == Some(&0x22) {
                a.push(quote(game, b, p)?);
            }
            a.extend(numbers(game, b, p, 1)?);
            a
        }
        (_, 0xc1 | 0xc2) => {
            let mut a = numbers(game, b, p, 1)?;
            a.push(quote(game, b, p)?);
            a
        }
        (_, 0xc6) => numbers(game, b, p, 5)?,
        (Game::Lipstick1, 0xc7) => Vec::new(),
        (_, 0xc7) => numbers(game, b, p, 1)?,
        (Game::Lipstick3, 0xc8 | 0xc9) => {
            let mut a = vec![quote(game, b, p)?];
            if code == 0xc8 {
                a.extend(numbers(game, b, p, 1)?);
            }
            a
        }
        (_, 0xc8) => {
            let mut a = vec![quote(game, b, p)?];
            a.extend(numbers(game, b, p, 1)?);
            a
        }
        (_, 0xc9) => {
            let mut a = Vec::new();
            while let Some(n) = number(game, b, p)? {
                a.push(Arg::Number(n));
            }
            a
        }
        (Game::Lipstick3, 0xca) => {
            let mut a = vec![variable(b, p)?];
            a.extend(numbers(game, b, p, 3)?);
            a
        }
        (Game::Lipstick3, 0xcb) => {
            let mut a = numbers(game, b, p, 1)?;
            if first(&a) == Some(2) {
                a.push(quote(game, b, p)?);
            } else if first(&a).is_some_and(|n| n <= 5) {
                a.extend(numbers(game, b, p, 1)?);
            }
            a
        }
        (_, 0xca | 0xcb | 0xce | 0xcf) => numbers(game, b, p, 1)?,
        (_, 0xcc) => numbers(game, b, p, 3)?,
        (Game::Lipstick3, 0xcd) => {
            let mut a = numbers(game, b, p, 1)?;
            loop {
                if b.get(*p) == Some(&0x22) {
                    a.push(quote(game, b, p)?);
                } else if let Some(n) = number(game, b, p)? {
                    a.push(Arg::Number(n));
                } else {
                    break;
                }
            }
            a
        }
        (_, 0xcd) => numbers(game, b, p, 4)?,
        (_, 0xd0) => {
            if game == Game::Lipstick3
                && b.get(*p..*p + 2)
                    .is_some_and(|v| v.eq_ignore_ascii_case(b"se"))
            {
                *p += 2;
                while b.get(*p) == Some(&b' ') {
                    *p += 1;
                }
            }
            if b.get(*p) == Some(&0x22) {
                vec![quote(game, b, p)?]
            } else {
                let mut a = numbers(game, b, p, 1)?;
                if game == Game::Lipstick3 && first(&a).is_some_and(|n| (1..=3).contains(&n)) {
                    a.extend(numbers(game, b, p, 1)?);
                }
                a
            }
        }
        (Game::Lipstick3, 0xd1) => {
            let mut a = numbers(game, b, p, 1)?;
            if let Some(n) = first(&a) {
                a.extend(numbers(game, b, p, if n == 2 { 2 } else { 1 })?);
            }
            a
        }
        (_, 0xd1) => {
            let mut a = numbers(game, b, p, 1)?;
            match first(&a) {
                Some(0 | 2) => {
                    let n = number(game, b, p)?;
                    if let Some(n) = n {
                        a.push(Arg::Number(n));
                        a.extend(numbers(game, b, p, 1)?);
                    }
                }
                Some(1 | 3 | 4) => a.extend(numbers(game, b, p, 1)?),
                _ => {}
            }
            a
        }
        (Game::Lipstick3, 0xd2) => Vec::new(),
        (_, 0xd2) => {
            let mut a = numbers(game, b, p, 1)?;
            if first(&a) != Some(4) {
                a.push(quote(game, b, p)?);
            }
            a
        }
        (_, 0xd3) => numbers(game, b, p, 2)?,
        (Game::Lipstick3, 0xd5) => numbers(game, b, p, 1)?,
        _ => Vec::new(),
    };
    if game == Game::Lipstick3 && code == 0xce {
        out.extend(numbers(game, b, p, 1)?);
    }
    if game == Game::Lipstick3 && code == 0xcf {
        out.extend(numbers(game, b, p, 3)?);
    }
    Ok(out)
}

pub fn lex(game: Game, source: &[u8]) -> Result<View> {
    let mut b = source.to_vec();
    let mut tokens = Vec::new();
    let mut p = 0;
    let mut encryption = None;
    let mut terminated = false;
    while p < b.len() {
        let start = p;
        let code = b[p];
        p += 1;
        let kind = match code {
            0x00..=0x1f => {
                let count = if game == Game::Lipstick3 && code >= 0x10 {
                    2
                } else {
                    1
                };
                b.get(p..p + count)
                    .ok_or_else(|| format!("0x{start:X}: 状态字截断"))?;
                p += count;
                Kind::State
            }
            0x21 if game == Game::Lipstick3 => {
                let end = b[p..]
                    .iter()
                    .position(|v| *v == 0)
                    .ok_or("21 字串缺少 NUL")?
                    + p;
                let text = raw_text(&b[p..end])?;
                p = end + 1;
                Kind::ZeroText(text)
            }
            0x2d..=0x7f if game != Game::Lipstick1 => Kind::Text(decode(&[0x82, code + 0x72])?),
            0xa0..=0xa4 => Kind::Boundary(u16::from(code)),
            0xa5 => Kind::Newline,
            0xff if b.get(p).is_some_and(|v| matches!(v, 0xff | 0xfe)) => {
                let value = u16::from_be_bytes([code, b[p]]);
                p += 1;
                if value == 0xfffe {
                    terminated = true;
                }
                Kind::Boundary(value)
            }
            c if is_lead(game, c) => {
                let second = *b.get(p).ok_or("日文双字节截断")?;
                p += 1;
                let value = u16::from_be_bytes([c, second]);
                if controls(game).contains(&value) {
                    Kind::Control(value)
                } else {
                    Kind::Text(glyph_text([c, second]))
                }
            }
            c if (0xa6..=match game {
                Game::Lipstick1 => 0xc7,
                Game::Lipstick2 => 0xd4,
                Game::Lipstick3 => 0xd8,
            })
                .contains(&c) =>
            {
                let args = operands(game, c, &b, &mut p)
                    .map_err(|e| format!("0x{start:X} {c:02X}: {e}"))?;
                if game == Game::Lipstick3
                    && c == 0xd5
                    && let Some(key) = first(&args)
                    && key != 0
                {
                    if key > 255 || encryption.is_some() {
                        return Err("未知的嵌套 D5 加密／密钥".into());
                    }
                    for value in &mut b[p..] {
                        *value = value.rotate_right(4) ^ key as u8;
                    }
                    encryption = Some(Encryption {
                        start: p,
                        key: key as u8,
                    });
                }
                Kind::Op(c, args)
            }
            _ => return Err(format!("0x{start:X}: 未确认的指令字节 {code:02X}")),
        };
        tokens.push(Token {
            span: Span { start, end: p },
            kind,
        });
        if terminated {
            break;
        }
    }
    if !terminated {
        return Err("缺少 FFFE 脚本终止标记".into());
    }
    Ok(View {
        bytes: b,
        tokens,
        script_end: p,
        encryption,
    })
}

fn definitions(game: Game, file: &str, view: &View) -> Result<Vec<Definition>> {
    let t = &view.tokens;
    let mut defs = Vec::new();
    let mut depth = 0usize;
    let mut i = 0;
    while i < t.len() {
        if let Kind::Op(0xb9, args) = &t[i].kind {
            let slot = first(args).ok_or("B9 缺少编号")?;
            if game == Game::Lipstick3 && slot >= 20 {
                return Err("三代 B9 槽编号超过 19".into());
            }
            if !matches!(t.get(i + 1).map(|x| &x.kind), Some(Kind::Boundary(0xa2))) {
                return Err("B9 缺少 A2".into());
            }
            let end = block_end(t, i + 1).ok_or("B9 块未闭合")?;
            let body_start = t[i + 1].span.end;
            let body_end = t[end].span.start;
            let body = view.bytes[body_start..body_end].to_vec();
            if game == Game::Lipstick3 && body.len() + 1 > 76 {
                return Err(format!("B9 槽 {slot} 超过 76 字节"));
            }
            let mut name = None;
            let visible = t[i + 2..end]
                .iter()
                .filter(|x| matches!(x.kind, Kind::Text(_)))
                .collect::<Vec<_>>();
            if game == Game::Lipstick3
                && visible.len() > 1
                && matches!(&visible.last().ok_or("空宏")?.kind, Kind::Text(s) if s == "「")
            {
                let letters = &visible[..visible.len() - 1];
                // The player's macro moves the window between its name and 「.
                // Keep that A6 outside the editable name span.
                if letters
                    .windows(2)
                    .all(|pair| pair[0].span.end == pair[1].span.start)
                {
                    let text = letters
                        .iter()
                        .filter_map(|x| {
                            if let Kind::Text(s) = &x.kind {
                                Some(s.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<String>();
                    name = Some((
                        text,
                        Span {
                            start: letters[0].span.start - body_start,
                            end: letters[letters.len() - 1].span.end - body_start,
                        },
                    ));
                }
            }
            let has_text = t[i + 2..end]
                .iter()
                .any(|x| matches!(x.kind, Kind::Text(_) | Kind::ZeroText(_)));
            defs.push(Definition {
                start: i,
                end,
                slot,
                depth,
                value: Macro {
                    file: file.into(),
                    offset: t[i].span.start,
                    body,
                    name,
                    has_text,
                },
            });
            i = end + 1;
            continue;
        }
        match t[i].kind {
            Kind::Boundary(0xa2) => depth += 1,
            Kind::Boundary(0xa3) => depth = depth.saturating_sub(1),
            _ => {}
        }
        i += 1;
    }
    Ok(defs)
}

fn block_end(tokens: &[Token], start: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, t) in tokens.iter().enumerate().skip(start) {
        match t.kind {
            Kind::Boundary(0xa2) => depth += 1,
            Kind::Boundary(0xa3) => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            Kind::Boundary(0xfffe) => return None,
            _ => {}
        }
    }
    None
}

pub fn common_macros(file: &str, bytes: &[u8]) -> Result<Macros> {
    let view = lex(Game::Lipstick3, bytes)?;
    let mut out = Macros::new();
    for d in definitions(Game::Lipstick3, file, &view)? {
        if d.depth == 0 {
            out.insert(d.slot, d.value);
        } else {
            out.remove(&d.slot);
        }
    }
    Ok(out)
}

fn player_view(text: &str, pattern: &[u8]) -> String {
    if let Ok(name) = decode(pattern)
        && !name.is_empty()
    {
        return text.replace(&name, "{player}");
    }
    text.into()
}

fn literal_name(tokens: &[Token], start: usize) -> Option<(usize, String, Span)> {
    if !matches!(&tokens[start].kind, Kind::Text(s) if s == "［") {
        return None;
    }
    let mut name = String::new();
    for (i, t) in tokens.iter().enumerate().skip(start + 1) {
        let Kind::Text(s) = &t.kind else {
            return None;
        };
        if s == "］" {
            return (!name.is_empty()).then(|| {
                (
                    i + 1,
                    name,
                    Span {
                        start: tokens[start].span.end,
                        end: t.span.start,
                    },
                )
            });
        }
        if s.contains('［') {
            return None;
        }
        name.push_str(s);
    }
    None
}

fn pending_end(tokens: &[Token], next: usize) -> Option<usize> {
    if matches!(
        tokens.get(next).map(|t| &t.kind),
        Some(Kind::Op(0xb2..=0xb5 | 0xbc | 0xbd, _))
    ) && matches!(
        tokens.get(next + 1).map(|t| &t.kind),
        Some(Kind::Boundary(0xa2))
    ) {
        return block_end(tokens, next + 1);
    }
    None
}

pub fn parse(
    game: Game,
    file: &str,
    source: &[u8],
    common: &Macros,
    player: &[u8],
) -> Result<Primary> {
    Ok(parse_with_view(game, file, source, common, player)?.0)
}

pub(crate) fn parse_with_view(
    game: Game,
    file: &str,
    source: &[u8],
    common: &Macros,
    player: &[u8],
) -> Result<(Primary, View)> {
    let view = lex(game, source).map_err(|e| format!("{file}: {e}"))?;
    let defs = definitions(game, file, &view)?;
    let defs_at: BTreeMap<_, _> = defs.iter().map(|d| (d.start, d)).collect();
    let t = &view.tokens;
    let mut macros = common.clone();
    let mut records = Vec::new();
    let mut pending: Option<(Speaker, Option<usize>)> = None;
    let mut i = 0;
    while i < t.len() {
        if pending
            .as_ref()
            .is_some_and(|(_, end)| end.is_some_and(|end| i >= end))
        {
            pending = None;
        }
        if matches!(t[i].kind, Kind::Op(0xb2..=0xb5 | 0xbc | 0xbd, _))
            && let Some((_, end)) = &mut pending
            && end.is_none()
        {
            *end = pending_end(t, i);
        }
        if let Some(d) = defs_at.get(&i) {
            if d.depth == 0 {
                macros.insert(d.slot, d.value.clone());
            } else {
                macros.remove(&d.slot);
            }
            if d.value.name.is_some() {
                i = d.end + 1;
                continue;
            }
        }
        if let Some((next, name, inner)) = literal_name(t, i) {
            pending = Some((
                Speaker {
                    name: player_view(&name, player),
                    origin: SpeakerOrigin::Literal { inner },
                },
                pending_end(t, next),
            ));
            i = next;
            continue;
        }
        if let Kind::Op(0xba, args) = &t[i].kind
            && let Some(slot) = first(args)
            && let Some(m) = macros.get(&slot)
            && let Some((name, inner)) = &m.name
        {
            pending = Some((
                Speaker {
                    name: player_view(name, player),
                    origin: SpeakerOrigin::Macro {
                        call: t[i].span.clone(),
                        slot,
                        definition_file: m.file.clone(),
                        definition_offset: m.offset,
                        body: m.body.clone(),
                        inner: inner.clone(),
                    },
                },
                pending_end(t, i + 1),
            ));
            i += 1;
            continue;
        }
        if let Kind::ZeroText(text) = &t[i].kind {
            let mut message = text.clone();
            let mut speaker = None;
            if let Some(rest) = message.strip_prefix('［')
                && let Some((name, body)) = rest.split_once('］')
                && !name.is_empty()
            {
                speaker = Some(Speaker {
                    name: player_view(name, player),
                    origin: SpeakerOrigin::ZeroString,
                });
                message = body.into();
            }
            if !message.trim().is_empty() {
                records.push(Record {
                    span: t[i].span.clone(),
                    mode: Mode::ZeroString,
                    entry: Entry {
                        name: speaker.as_ref().map(|s| s.name.clone()),
                        message: player_view(&message, player),
                    },
                    speaker,
                });
            }
            pending = None;
            i += 1;
            continue;
        }
        let piece = |token: &Token, macros: &Macros| -> Option<String> {
            match &token.kind {
                Kind::Text(s) => Some(s.clone()),
                Kind::Newline => Some("\n".into()),
                Kind::Op(0xba, args) => first(args).and_then(|slot| {
                    // Keep text macros as calls, including dynamically redefined player honorifics.
                    if (game == Game::Lipstick3 && slot == 9)
                        || macros
                            .get(&slot)
                            .is_some_and(|m| m.has_text && m.name.is_none())
                    {
                        Some(format!("{{macro:{slot}}}"))
                    } else {
                        None
                    }
                }),
                _ => None,
            }
        };
        if piece(&t[i], &macros).is_some_and(|s| !s.trim().is_empty()) {
            let start = i;
            let mut message = String::new();
            while i < t.len() {
                if i != start && literal_name(t, i).is_some() {
                    break;
                }
                let Some(s) = piece(&t[i], &macros) else {
                    break;
                };
                message.push_str(&s);
                i += 1;
            }
            while i > start && matches!(t[i - 1].kind, Kind::Newline) {
                message.pop();
                i -= 1;
            }
            if i == start {
                i += 1;
                continue;
            }
            let speaker = pending.as_ref().map(|(s, _)| s.clone());
            if !message.trim().is_empty() {
                records.push(Record {
                    span: Span {
                        start: t[start].span.start,
                        end: t[i - 1].span.end,
                    },
                    mode: Mode::Script,
                    entry: Entry {
                        name: speaker.as_ref().map(|s| s.name.clone()),
                        message: player_view(&message, player),
                    },
                    speaker,
                });
            }
            if pending.as_ref().is_some_and(|(_, end)| end.is_none()) {
                pending = None;
            }
            continue;
        }
        if matches!(
            t[i].kind,
            Kind::Boundary(0xffff | 0xfffe)
                | Kind::Op(0xa6 | 0xa7 | 0xaa | 0xad | 0xae | 0xb1 | 0xc0 | 0xd4, _)
        ) {
            pending = None;
        }
        i += 1;
    }
    let opens = t.iter().filter(|x| x.kind == Kind::Boundary(0xa2)).count();
    let closes = t.iter().filter(|x| x.kind == Kind::Boundary(0xa3)).count();
    let mut diagnostics = Vec::new();
    if opens != closes {
        diagnostics.push(format!(
            "原始块计数不平衡：A2={opens}, A3={closes}；保留原结构"
        ));
    }
    let native = records
        .iter()
        .filter(|r| r.entry.message.contains("{glyph:"))
        .count();
    if native != 0 {
        diagnostics.push(format!("{native} 条含按原字节保留的原生／自定义字形占位符"));
    }
    Ok((
        Primary {
            file: file.into(),
            game,
            original: source.to_vec(),
            player_pattern: player.to_vec(),
            records,
            diagnostics,
        },
        view,
    ))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    Text(String),
    Player,
    Glyph(u16),
    Macro(u16),
}

pub fn parts(text: &str) -> Result<Vec<Part>> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        if start != 0 {
            out.push(Part::Text(rest[..start].into()));
        }
        let end = rest[start..]
            .find('}')
            .ok_or("占位符缺少 }（普通花括号请用全角）")?
            + start;
        let marker = &rest[start + 1..end];
        let value = if marker == "player" {
            Part::Player
        } else if let Some(code) = marker.strip_prefix("glyph:") {
            if code.len() != 4 {
                return Err("glyph 码位必须为四位十六进制".into());
            }
            Part::Glyph(u16::from_str_radix(code, 16).map_err(|_| "glyph 码位非法")?)
        } else if let Some(slot) = marker.strip_prefix("macro:") {
            Part::Macro(slot.parse().map_err(|_| "macro 编号非法")?)
        } else {
            return Err(format!("未知占位符 {{{marker}}}"));
        };
        out.push(value);
        rest = &rest[end + 1..];
    }
    if rest.contains('}') {
        return Err("多余的占位符结束符 }".into());
    }
    if !rest.is_empty() {
        out.push(Part::Text(rest.into()));
    }
    Ok(out)
}

fn signature(s: &str) -> Result<Vec<Part>> {
    Ok(parts(s)?
        .into_iter()
        .filter(|p| !matches!(p, Part::Text(_)))
        .collect())
}

pub fn normalize(text: &str, ascii: bool) -> Result<String> {
    let mut out = String::new();
    for c in text.chars() {
        if c == '\n' {
            out.push(c);
        } else if c.is_control() {
            return Err(format!("不支持控制字符 U+{:04X}", c as u32));
        } else if !ascii && c == ' ' {
            out.push('　');
        } else if !ascii && ('!'..='~').contains(&c) {
            out.push(char::from_u32(c as u32 + 0xfee0).ok_or("全角化失败")?);
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

pub fn changed_texts(primary: &Primary, supplied: &[Entry]) -> Result<Vec<String>> {
    let expected = primary
        .records
        .iter()
        .map(|r| r.entry.clone())
        .collect::<Vec<_>>();
    vn_text::validate_shape(&expected, supplied).map_err(|e| e.to_string())?;
    let mut texts = Vec::new();
    for (index, (record, entry)) in primary.records.iter().zip(supplied).enumerate() {
        for (old, new, name) in [
            (record.entry.message.as_str(), entry.message.as_str(), false),
            (
                record.entry.name.as_deref().unwrap_or(""),
                entry.name.as_deref().unwrap_or(""),
                true,
            ),
        ] {
            if old == new {
                continue;
            }
            if signature(old)? != signature(new)? {
                return Err(format!(
                    "{} 第 {} 条：玩家名、字形或宏占位符被更改／乱序",
                    primary.file,
                    index + 1
                ));
            }
            if name && (new.is_empty() || new.contains(['\n', '\r', '［', '］'])) {
                return Err("name 不能为空、带括号或换行".into());
            }
            for part in parts(new)? {
                if let Part::Text(s) = part {
                    texts.push(normalize(&s, record.mode == Mode::ZeroString)?.replace('\n', ""));
                }
            }
        }
    }
    Ok(texts)
}

pub fn encode_field(
    primary: &Primary,
    text: &str,
    mode: Mode,
    encode: &impl Fn(&str, bool) -> Result<Vec<u8>>,
) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for p in parts(text)? {
        match p {
            Part::Player => out.extend(&primary.player_pattern),
            Part::Glyph(code) => out.extend(code.to_be_bytes()),
            Part::Macro(slot) => {
                if mode != Mode::Script {
                    return Err("原字串内不能插入 BA 指令".into());
                }
                out.push(0xba);
                out.extend(encode_number(primary.game, slot));
            }
            Part::Text(s) => {
                let value = normalize(&s, mode == Mode::ZeroString)?;
                for (i, line) in value.split('\n').enumerate() {
                    if i != 0 {
                        out.push(if mode == Mode::Script { 0xa5 } else { b'\r' });
                    }
                    out.extend(encode(line, mode == Mode::ZeroString)?);
                }
            }
        }
    }
    Ok(out)
}

pub fn rebuild(
    primary: &Primary,
    supplied: &[Entry],
    encode: &impl Fn(&str, bool) -> Result<Vec<u8>>,
) -> Result<Vec<u8>> {
    if primary
        .records
        .iter()
        .zip(supplied)
        .all(|(r, e)| r.entry == *e)
        && primary.records.len() == supplied.len()
    {
        return Ok(primary.original.clone());
    }
    let view = lex(primary.game, &primary.original)?;
    rebuild_view(primary, &view, supplied, encode)
}

pub(crate) fn rebuild_view(
    primary: &Primary,
    view: &View,
    supplied: &[Entry],
    encode: &impl Fn(&str, bool) -> Result<Vec<u8>>,
) -> Result<Vec<u8>> {
    changed_texts(primary, supplied)?;
    if primary
        .records
        .iter()
        .zip(supplied)
        .all(|(r, e)| r.entry == *e)
    {
        return Ok(primary.original.clone());
    }
    let mut patches = BTreeMap::<(usize, usize), Vec<u8>>::new();
    let mut speakers = BTreeMap::<(usize, usize), String>::new();
    for (r, entry) in primary.records.iter().zip(supplied) {
        if let Some(speaker) = &r.speaker {
            let key = match &speaker.origin {
                SpeakerOrigin::Literal { inner } => Some((inner.start, inner.end)),
                SpeakerOrigin::Macro { call, .. } => Some((call.start, call.end)),
                SpeakerOrigin::ZeroString => None,
            };
            if let Some(key) = key {
                let name = entry.name.clone().ok_or("缺少 name")?;
                if speakers
                    .insert(key, name.clone())
                    .is_some_and(|previous| previous != name)
                {
                    return Err("同一个姓名头用于多个分支，请为这些条目提供一致的 name".into());
                }
            }
        }
        let whole_zero = r.mode == Mode::ZeroString
            && matches!(
                r.speaker.as_ref().map(|s| &s.origin),
                Some(SpeakerOrigin::ZeroString)
            );
        if r.entry.message != entry.message || (whole_zero && r.entry.name != entry.name) {
            let message = if whole_zero {
                format!(
                    "［{}］{}",
                    entry.name.as_deref().ok_or("缺少 name")?,
                    entry.message
                )
            } else {
                entry.message.clone()
            };
            let mut bytes = encode_field(primary, &message, r.mode, encode)?;
            if r.mode == Mode::ZeroString {
                bytes.insert(0, 0x21);
                bytes.push(0);
            }
            patches.insert((r.span.start, r.span.end), bytes);
        }
        if let Some(speaker) = &r.speaker
            && entry.name.as_deref() != Some(speaker.name.as_str())
        {
            let name = entry.name.as_deref().ok_or("缺少 name")?;
            let bytes = encode_field(primary, name, Mode::Script, encode)?;
            match &speaker.origin {
                SpeakerOrigin::Literal { inner } => {
                    patches.insert((inner.start, inner.end), bytes);
                }
                SpeakerOrigin::Macro {
                    call, body, inner, ..
                } => {
                    let mut replacement = body.get(..inner.start).ok_or("宏姓名起点越界")?.to_vec();
                    replacement.extend(bytes);
                    replacement.extend_from_slice(body.get(inner.end..).ok_or("宏姓名终点越界")?);
                    patches.insert((call.start, call.end), replacement);
                }
                SpeakerOrigin::ZeroString => {}
            }
        }
    }
    let fixed = view.encryption.is_some() || view.script_end < view.bytes.len();
    let mut out = Vec::new();
    let mut cursor = 0;
    for ((start, end), mut bytes) in patches {
        if start < cursor || end < start || end > view.script_end {
            return Err("文本定位重叠或越过原生附录".into());
        }
        if fixed {
            if bytes.len() > end - start {
                return Err(format!(
                    "{}: 带原生附录的字段最多 {} 字节，译文 {} 字节",
                    primary.file,
                    end - start,
                    bytes.len()
                ));
            }
            if primary.game != Game::Lipstick3 {
                return Err("未知固定布局脚本".into());
            }
            bytes.resize(end - start, 0xd6);
        }
        out.extend_from_slice(&view.bytes[cursor..start]);
        out.extend(bytes);
        cursor = end;
    }
    out.extend_from_slice(&view.bytes[cursor..]);
    // These are new-output boundary checks, not a second pass over old images/hashes.
    let parsed = if let Some(enc) = &view.encryption {
        let mut wrapped = out.clone();
        for value in &mut wrapped[enc.start..] {
            *value = (*value ^ enc.key).rotate_left(4);
        }
        lex(primary.game, &wrapped)?
    } else {
        lex(primary.game, &out)?
    };
    definitions(primary.game, &primary.file, &parsed)?;
    if let Some(enc) = &view.encryption {
        for value in &mut out[enc.start..] {
            *value = (*value ^ enc.key).rotate_left(4);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cp(s: &str, _: bool) -> Result<Vec<u8>> {
        direct(s).ok_or("encoding".into())
    }
    #[test]
    fn literal_name_and_branch_group_share_the_physical_header() {
        let mut b = direct("［音美］").unwrap();
        b.extend([0xb2, 0xa2]);
        b.extend(direct("日本").unwrap());
        b.push(0xa4);
        b.extend(direct("世界").unwrap());
        b.extend([0xa3, 0xff, 0xff, 0xff, 0xfe]);
        let p = parse(Game::Lipstick2, "test", &b, &Macros::new(), &[]).unwrap();
        assert_eq!(p.records.len(), 2);
        assert!(
            p.records
                .iter()
                .all(|r| r.entry.name.as_deref() == Some("音美"))
        );
        let mut edits = p
            .records
            .iter()
            .map(|r| r.entry.clone())
            .collect::<Vec<_>>();
        edits[0].name = Some("甲".into());
        assert!(rebuild(&p, &edits, &cp).is_err());
    }
    #[test]
    fn third_generation_variables_and_native_glyph_are_not_text() {
        let b = [0x10, 0x2e, 0, 0xeb, 0x9f, 0x45, 0xff, 0xff, 0xff, 0xfe];
        let p = parse(Game::Lipstick3, "t", &b, &Macros::new(), &[]).unwrap();
        assert_eq!(p.records[0].entry.message, "{glyph:EB9F}す");
        let mut e = vec![p.records[0].entry.clone()];
        e[0].message = "日本".into();
        assert!(rebuild(&p, &e, &cp).is_err());
        assert!(lex(Game::Lipstick3, &[0x10, 0x2e]).is_err());
    }

    #[test]
    fn advg_resource_argument_owns_its_comma_and_quote() {
        let b = b"\xc1"
            .iter()
            .copied()
            .chain(b"0,\"ANIME.MDA\"".iter().copied())
            .chain([255, 255, 255, 254])
            .collect::<Vec<_>>();
        let view = lex(Game::Lipstick1, &b).unwrap();
        assert!(matches!(&view.tokens[0].kind, Kind::Op(0xc1, a) if a.len() == 2));
    }
    #[test]
    fn macro_name_uses_current_definition_and_preserves_the_open_quote() {
        let mut b = vec![0xb9, 0x24, 0xa2];
        b.extend(direct("音美「").unwrap());
        b.push(0xa3);
        b.extend([0xba, 0x24]);
        b.extend(direct("日本」").unwrap());
        b.extend([0xff, 0xff, 0xff, 0xfe]);
        let p = parse(Game::Lipstick3, "t", &b, &Macros::new(), &[]).unwrap();
        assert_eq!(p.records.len(), 1);
        assert_eq!(p.records[0].entry.name.as_deref(), Some("音美"));
        let mut e = vec![p.records[0].entry.clone()];
        e[0].name = Some("甲".into());
        let rebuilt = rebuild(&p, &e, &cp).unwrap();
        assert!(rebuilt.windows(4).any(|v| v == direct("甲「").unwrap()));
    }

    #[test]
    fn player_macro_can_move_the_window_before_printing_the_quote() {
        let marker = direct("＊＊＊").unwrap();
        let mut b = vec![0xb9, 0x23, 0xa2];
        b.extend(&marker);
        b.extend([0xa6, 0x23, 0x23, 0x24, 0x24]);
        b.extend(direct("「").unwrap());
        b.extend([0xa3, 0xba, 0x23]);
        b.extend(direct("日本」").unwrap());
        b.extend([255, 255, 255, 254]);
        let p = parse(Game::Lipstick3, "player", &b, &Macros::new(), &marker).unwrap();
        assert_eq!(p.records[0].entry.name.as_deref(), Some("{player}"));
        assert_eq!(p.records[0].entry.message, "日本」");
    }
}
