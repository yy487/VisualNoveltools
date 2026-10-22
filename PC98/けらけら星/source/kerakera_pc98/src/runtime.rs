use crate::mes::Result;

fn word(b: &[u8], p: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(p..p + 2)
            .ok_or("MZ 字段越界")?
            .try_into()
            .map_err(|_| "MZ 字段长度")?,
    ))
}

/// This game's TC0 and EXEPACK startup, retained only for version checks and
/// conservative protection of glyphs referenced by untouched native routines.
pub fn module(source: &[u8]) -> Result<Vec<u8>> {
    let mut b = source.to_vec();
    for _ in 0..32 {
        if b.get(..2) != Some(b"MZ") {
            return Err("运行模块不是 MZ".into());
        }
        let header = usize::from(word(&b, 8)?) * 16;
        let entry = header + usize::from(word(&b, 0x16)?) * 16 + usize::from(word(&b, 0x14)?);
        if b.get(entry..entry + 1) != Some(&[0xe8]) || b.get(entry + 3..entry + 6) != Some(b"TC0") {
            break;
        }
        let delta = word(&b, entry + 1)? as i16;
        let stub = (entry + 3)
            .checked_add_signed(isize::from(delta))
            .ok_or("TC0 stub 越界")?;
        if b.get(stub..stub + 4) != Some(&[0x5b, 0x83, 0xc3, 3]) {
            return Err("未知 TC0 stub".into());
        }
        let mut state = word(&b, entry + 6)?;
        let count = usize::from(word(&b, entry + 8)?);
        let di = usize::from(word(&b, entry + 10)?);
        let start = header
            .checked_add(di.checked_sub(0x100).ok_or("TC0 目标位于 PSP")?)
            .ok_or("TC0 地址溢出")?;
        if start < header || start + count > entry || entry >= b.len() {
            return Err("TC0 数据范围非法".into());
        }
        let ip = word(&b, entry + 12)?;
        let cs = word(&b, entry + 14)?;
        for value in &mut b[start..start + count] {
            state = state.wrapping_mul(0x2711).wrapping_add(0x3619);
            *value ^= (state.rotate_left(2) >> 8) as u8;
        }
        let body = b.get(header..).ok_or("MZ 头越界")?.to_vec();
        b.truncate(32);
        b[8..10].copy_from_slice(&2u16.to_le_bytes());
        b[0x14..0x16].copy_from_slice(&ip.to_le_bytes());
        b[0x16..0x18].copy_from_slice(&cs.to_le_bytes());
        b.extend(body);
    }
    let header = usize::from(word(&b, 8)?) * 16;
    let pack = usize::from(word(&b, 0x16)?) * 16;
    let body = b.get(header..).ok_or("MZ 头越界")?;
    if body.get(pack + 14..pack + 16) != Some(b"RB") {
        return Ok(body.to_vec());
    }
    let mut output = vec![0; usize::from(word(body, pack + 12)?) * 16];
    let mut s = pack;
    while s > 0 && body[s - 1] == 0xff {
        s -= 1;
    }
    let mut d = output.len();
    for _ in 0..65536 {
        if s < 3 {
            return Err("EXEPACK 命令截断".into());
        }
        let command = body[s - 1];
        let count = usize::from(word(body, s - 3)?);
        s -= 3;
        let dest = d.checked_sub(count).ok_or("EXEPACK 输出越界")?;
        match command & 0xfe {
            0xb0 => {
                s = s.checked_sub(1).ok_or("EXEPACK 填充字节缺失")?;
                output[dest..d].fill(body[s]);
            }
            0xb2 => {
                let start = s.checked_sub(count).ok_or("EXEPACK 数据缺失")?;
                output[dest..d].copy_from_slice(&body[start..s]);
                s = start;
            }
            _ => return Err("未知 EXEPACK 命令".into()),
        }
        d = dest;
        if command & 1 != 0 {
            if d != s {
                return Err("EXEPACK 原始前缀范围不匹配".into());
            }
            output[..d].copy_from_slice(&body[..s]);
            return Ok(output);
        }
    }
    Err("EXEPACK 命令过多".into())
}

pub fn ini(source: &[u8]) -> Result<Vec<u8>> {
    if source.get(0x50..0x53) != Some(&[0xd5, 0x28, 0x4b]) {
        return Err("不是已确认的本作 INI 启动脚本".into());
    }
    let mut b = source.to_vec();
    for value in &mut b[0x53..] {
        *value = value.rotate_right(4) ^ 0x4b;
    }
    Ok(b)
}

pub fn validate_engine(source: &[u8]) -> Result<Vec<u8>> {
    let b = module(source)?;
    let expected: [u16; 51] = [
        0xdf87, 0xdfbb, 0xdfd0, 0xdfee, 0xe001, 0xe01a, 0xe02d, 0xe03e, 0xe3c7, 0xe477, 0xe4ef,
        0xe596, 0xe5b8, 0xe5e2, 0xe5f2, 0xe604, 0xe617, 0xe65b, 0xe6ad, 0xe6c4, 0xe6db, 0xe6ee,
        0xe704, 0xe71b, 0xe726, 0xe74f, 0xe7e5, 0xe839, 0xe855, 0xe863, 0xe871, 0xe889, 0xe89c,
        0xe8b8, 0xe8fd, 0xe915, 0xe929, 0xe93e, 0xe94c, 0xe99d, 0xea2e, 0xea46, 0xea84, 0xeb5f,
        0xebde, 0xec10, 0xe824, 0xeb11, 0xec1d, 0xec1d, 0xec1d,
    ];
    for (i, value) in expected.iter().enumerate() {
        if word(&b, 0xd8c0 + i * 2)? != *value {
            return Err("ADV98V 分发表与本作已分析版本不符".into());
        }
    }
    Ok(b)
}
