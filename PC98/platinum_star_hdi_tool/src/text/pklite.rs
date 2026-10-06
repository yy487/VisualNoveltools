enum Huffman {
    Leaf(i16),
    Branch(Box<Huffman>, Box<Huffman>),
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    word: u16,
    bits: u8,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Result<Self, String> {
        if data.len() < 2 {
            return Err("PKLITE 压缩流缺少初始位字".into());
        }
        Ok(Self {
            data,
            pos: 2,
            word: u16::from_le_bytes([data[0], data[1]]),
            bits: 0,
        })
    }

    fn byte(&mut self) -> Result<u8, String> {
        let value = *self
            .data
            .get(self.pos)
            .ok_or_else(|| format!("PKLITE 字节读取越界 @0x{:X}", self.pos))?;
        self.pos += 1;
        Ok(value)
    }

    fn bit(&mut self) -> Result<u8, String> {
        let value = ((self.word >> self.bits) & 1) as u8;
        self.bits += 1;
        if self.bits == 16 {
            self.bits = 0;
            let low = self.byte()?;
            let high = self.byte()?;
            self.word = u16::from_le_bytes([low, high]);
        }
        Ok(value)
    }

    fn tree(&mut self, tree: &Huffman) -> Result<i16, String> {
        let mut node = tree;
        loop {
            match node {
                Huffman::Leaf(value) => return Ok(*value),
                Huffman::Branch(left, right) => {
                    node = if self.bit()? == 0 { left } else { right };
                }
            }
        }
    }
}

pub fn unpack_mz_file(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 0x1E || !matches!(&data[..2], b"MZ" | b"ZM") {
        return Err("输入不是 DOS MZ 文件".into());
    }
    let original_header_paras = u16_at(data, 8)? as usize;
    let code_start = original_header_paras * 16;
    let hint = *data
        .get(code_start + 0x4E)
        .ok_or_else(|| "PKLITE 壳缺少压缩流位置提示".to_string())? as usize;
    if hint < 0x10 {
        return Err("PKLITE 压缩流位置提示无效".into());
    }
    let mut compressed_start = (original_header_paras + hint - 0x10) * 16;
    if compressed_start >= data.len() {
        return Err("PKLITE 压缩流位于文件末尾之后".into());
    }

    let work = descramble(data)?;
    let image_limit = u16_at(&work, code_start + 2)? as usize * 16 + 0x100;
    let signature = [0xFD, 0x8C, 0xDB, 0x53, 0x83, 0xC3];
    let search_end = work.len().min(compressed_start.saturating_add(0x40));
    if let Some(relative) = work[code_start..search_end]
        .windows(signature.len())
        .position(|window| window == signature)
    {
        let signature_pos = code_start + relative;
        let delta = *work
            .get(signature_pos + 6)
            .ok_or_else(|| "PKLITE 解压器流指针被截断".to_string())? as usize;
        compressed_start = code_start + delta * 16 - 0x100;
        if compressed_start < code_start || compressed_start >= work.len() {
            return Err("PKLITE 1.50 压缩流指针越界".into());
        }
    }

    let extra = work[0x1D] & 0x10 != 0;
    let length_tree = length_tree();
    let distance_tree = distance_tree();
    let mut reader = Reader::new(&work[compressed_start..])?;
    let mut image = Vec::new();
    loop {
        if reader.bit()? == 0 {
            let mut raw = reader.byte()?;
            if extra {
                raw ^= 16u8.wrapping_sub(reader.bits);
            }
            image.push(raw);
            if image.len() > image_limit {
                return Err("PKLITE 输出超过壳内图像上限".into());
            }
            continue;
        }
        let mut length = reader.tree(&length_tree)?;
        if length == 25 {
            let extension = reader.byte()?;
            if extension == 0xFE {
                continue;
            }
            if extension == 0xFF {
                break;
            }
            length = extension as i16 + 25;
        }
        if length < 2 {
            return Err(format!("PKLITE 回溯长度无效: {length}"));
        }
        let high = if length != 2 {
            reader.tree(&distance_tree)? as usize
        } else {
            0
        };
        let low = reader.byte()? as usize;
        let distance = (high << 8) | low;
        if image.len() + length as usize > image_limit {
            return Err("PKLITE 回溯超过壳内图像上限".into());
        }
        let start = image.len() as isize - distance as isize;
        for index in 0..length as usize {
            let source = start + index as isize;
            let value = if source >= 0 && (source as usize) < image.len() {
                image[source as usize]
            } else {
                0
            };
            image.push(value);
        }
    }
    let packed_end = compressed_start + reader.pos;
    rebuild_mz(&work, &image, packed_end)
}

fn descramble(data: &[u8]) -> Result<Vec<u8>, String> {
    let header_paras = u16_at(data, 8)? as usize;
    let base = header_paras * 16;
    if base + 7 > data.len() || data[base] != 0x50 || data[base + 4] != 0xBA {
        return Err("PKLITE 1.50 入口序言不匹配".into());
    }
    let initial_key = u16_at(data, base + 5)?;
    let jump_pos = base + 0x0E;
    if data.get(jump_pos) != Some(&0x72) {
        return Err("没有找到 PKLITE 1.50 扰码器跳转".into());
    }
    let descrambler = jump_pos + 2 + data[jump_pos + 1] as usize;
    if data.get(descrambler..descrambler + 6) != Some(&[0x59, 0x2D, 0x20, 0x00, 0x8E, 0xD0]) {
        return Err("PKLITE 1.50 扰码器签名不匹配".into());
    }
    let count = u16_at(data, descrambler + 20)? as usize;
    let last_ip = u16_at(data, descrambler + 23)? as usize;
    let last = base
        .checked_add(last_ip)
        .and_then(|value| value.checked_sub(0x100))
        .ok_or_else(|| "PKLITE 扰码范围下溢".to_string())?;
    if count < 2 || last < base || last + 2 > data.len() {
        return Err("PKLITE 1.50 扰码范围无效".into());
    }
    let start = (last + 2)
        .checked_sub((count - 1) * 2)
        .ok_or_else(|| "PKLITE 扰码起点下溢".to_string())?;
    if start < base || start + count * 2 > data.len() {
        return Err("PKLITE 1.50 扰码范围越界".into());
    }
    let mut work = data.to_vec();
    for pos in (start..last + 2).step_by(2) {
        let first = u16_at(&work, pos)?;
        let second = if pos == last {
            initial_key
        } else {
            u16_at(&work, pos + 2)?
        };
        work[pos..pos + 2].copy_from_slice(&(first ^ second).to_le_bytes());
    }
    Ok(work)
}

fn rebuild_mz(data: &[u8], image: &[u8], packed_end: usize) -> Result<Vec<u8>, String> {
    let (relocations, footer) = parse_mz_tail(data, packed_end)?;
    let extra = data[0x1D] & 0x10 != 0;
    let header_extra = if extra { &data[0x1C..0x1E] } else { &[] };
    let reloc_offset = 0x1C + header_extra.len();
    let header_paras = (reloc_offset + relocations.len() * 4).div_ceil(16);
    let mut output = vec![0u8; header_paras * 16];
    output[..2].copy_from_slice(b"MZ");
    put_u16(&mut output, 6, relocations.len() as u16);
    put_u16(&mut output, 8, header_paras as u16);
    let original_code = u16_at(data, 8)? as usize * 16;
    let image_limit = u16_at(data, original_code + 2)? as usize * 16 + 0x100;
    let min_extra = image_limit.saturating_sub(image.len()).div_ceil(16);
    put_u16(&mut output, 0x0A, min_extra as u16);
    put_u16(&mut output, 0x0C, 0xFFFF);
    put_u16(&mut output, 0x0E, footer[0]);
    put_u16(&mut output, 0x10, footer[1]);
    put_u16(&mut output, 0x14, footer[3]);
    put_u16(&mut output, 0x16, footer[2]);
    put_u16(&mut output, 0x18, reloc_offset as u16);
    output[0x1C..0x1C + header_extra.len()].copy_from_slice(header_extra);
    let mut pos = reloc_offset;
    for relocation in relocations {
        output[pos..pos + 4].copy_from_slice(&relocation.to_le_bytes());
        pos += 4;
    }
    output.extend_from_slice(image);
    let pages = output.len().div_ceil(512);
    let last = output.len() % 512;
    put_u16(&mut output, 4, pages as u16);
    put_u16(&mut output, 2, last as u16);
    Ok(output)
}

fn parse_mz_tail(data: &[u8], packed_end: usize) -> Result<(Vec<u32>, [u16; 4]), String> {
    let version = u16::from_le_bytes([data[0x1C], data[0x1D]]);
    let large = data[0x1D] & 0x10 != 0 && (version & 0x0FFF) >= 0x010C;
    let mut pos = packed_end;
    let mut relocations = Vec::new();
    let mut high = -0x0FFFi32;
    loop {
        let count = if large {
            let count = u16_at(data, pos)?;
            pos += 2;
            if count == 0xFFFF {
                break;
            }
            high += 0x0FFF;
            count as usize
        } else {
            let count = *data
                .get(pos)
                .ok_or_else(|| "PKLITE 重定位块被截断".to_string())?
                as usize;
            pos += 1;
            if count == 0 {
                break;
            }
            high = u16_at(data, pos)? as i32;
            pos += 2;
            count
        };
        for _ in 0..count {
            let low = u16_at(data, pos)? as u32;
            pos += 2;
            if high < 0 {
                return Err("PKLITE 重定位高位为负".into());
            }
            relocations.push(((high as u32) << 16) | low);
        }
    }
    let footer = [
        u16_at(data, pos)?,
        u16_at(data, pos + 2)?,
        u16_at(data, pos + 4)?,
        u16_at(data, pos + 6)?,
    ];
    Ok((relocations, footer))
}

fn length_tree() -> Huffman {
    use Huffman::{Branch as B, Leaf as L};
    fn b(left: Huffman, right: Huffman) -> Huffman {
        B(Box::new(left), Box::new(right))
    }
    b(
        b(
            b(L(4), b(L(5), L(6))),
            b(
                b(L(7), b(L(8), L(9))),
                b(
                    b(L(10), b(L(11), L(12))),
                    b(
                        b(L(25), b(L(13), L(14))),
                        b(
                            b(L(15), b(L(16), L(17))),
                            b(
                                b(L(18), b(L(19), L(20))),
                                b(b(L(21), L(22)), b(L(23), L(24))),
                            ),
                        ),
                    ),
                ),
            ),
        ),
        b(L(2), L(3)),
    )
}

fn distance_tree() -> Huffman {
    use Huffman::{Branch as B, Leaf as L};
    fn b(left: Huffman, right: Huffman) -> Huffman {
        B(Box::new(left), Box::new(right))
    }
    b(
        b(
            b(b(L(1), L(2)), b(b(L(3), L(4)), b(L(5), L(6)))),
            b(
                b(
                    b(b(L(7), L(8)), b(L(9), L(10))),
                    b(b(L(11), L(12)), b(L(13), b(L(14), L(15)))),
                ),
                b(
                    b(
                        b(b(L(16), L(17)), b(L(18), L(19))),
                        b(b(L(20), L(21)), b(L(22), L(23))),
                    ),
                    b(
                        b(b(L(24), L(25)), b(L(26), L(27))),
                        b(b(L(28), L(29)), b(L(30), L(31))),
                    ),
                ),
            ),
        ),
        L(0),
    )
}

fn u16_at(data: &[u8], pos: usize) -> Result<u16, String> {
    let raw = data
        .get(pos..pos + 2)
        .ok_or_else(|| format!("16 位读取越界 @0x{pos:X}"))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn put_u16(data: &mut [u8], pos: usize, value: u16) {
    data[pos..pos + 2].copy_from_slice(&value.to_le_bytes());
}
