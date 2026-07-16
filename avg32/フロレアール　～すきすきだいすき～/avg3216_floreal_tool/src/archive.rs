use std::collections::HashMap;

use anyhow::{Context, Result, bail, ensure};

const PACL_HEADER_SIZE: usize = 0x20;
const PACL_ENTRY_SIZE: usize = 0x20;
const PACK_HEADER_SIZE: usize = 0x10;

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

fn to_u32(value: usize, label: &str) -> Result<u32> {
    u32::try_from(value).with_context(|| format!("{label} 超过 u32：0x{value:X}"))
}

#[derive(Debug, Clone)]
pub struct PaclItem {
    pub index: usize,
    pub name: String,
    pub offset: usize,
    pub packed_size: usize,
    pub unpacked_size: usize,
    pub flag: u32,
    pub block: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PaclArchive {
    table: Vec<u8>,
    pub items: Vec<PaclItem>,
}

impl PaclArchive {
    pub fn parse(data: &[u8]) -> Result<Self> {
        ensure!(data.get(..4) == Some(b"PACL"), "文件不是 PACL");
        let count = read_u32(data, 0x10)? as usize;
        let table_size = PACL_HEADER_SIZE
            .checked_add(
                count
                    .checked_mul(PACL_ENTRY_SIZE)
                    .context("PACL 表大小溢出")?,
            )
            .context("PACL 表大小溢出")?;
        ensure!(table_size <= data.len(), "PACL 目录越过文件末尾");

        let mut items = Vec::with_capacity(count);
        let mut expected_offset = table_size;
        for index in 0..count {
            let entry = PACL_HEADER_SIZE + index * PACL_ENTRY_SIZE;
            let raw_name = &data[entry..entry + 0x10];
            let name_end = raw_name
                .iter()
                .position(|&byte| byte == 0)
                .unwrap_or(raw_name.len());
            let name = std::str::from_utf8(&raw_name[..name_end])
                .with_context(|| format!("PACL 条目 {index} 文件名不是 ASCII"))?
                .to_owned();
            let offset = read_u32(data, entry + 0x10)? as usize;
            let packed_size = read_u32(data, entry + 0x14)? as usize;
            let unpacked_size = read_u32(data, entry + 0x18)? as usize;
            let flag = read_u32(data, entry + 0x1C)?;
            ensure!(
                offset == expected_offset,
                "{name}: PACL 数据不连续：实际 0x{offset:X}，预期 0x{expected_offset:X}"
            );
            let end = offset
                .checked_add(packed_size)
                .with_context(|| format!("{name}: PACK 范围溢出"))?;
            let block = data
                .get(offset..end)
                .with_context(|| format!("{name}: PACK 越过文件末尾"))?
                .to_vec();
            ensure!(block.get(..4) == Some(b"PACK"), "{name}: 数据块不是 PACK");
            ensure!(
                read_u32(&block, 8)? as usize == unpacked_size,
                "{name}: PACL/PACK 解压尺寸不一致"
            );
            ensure!(
                read_u32(&block, 12)? as usize == packed_size,
                "{name}: PACL/PACK 压缩尺寸不一致"
            );
            items.push(PaclItem {
                index,
                name,
                offset,
                packed_size,
                unpacked_size,
                flag,
                block,
            });
            expected_offset = end;
        }
        ensure!(
            expected_offset == data.len(),
            "PACL 存在尾随数据：数据结束 0x{expected_offset:X}，文件结束 0x{:X}",
            data.len()
        );

        Ok(Self {
            table: data[..table_size].to_vec(),
            items,
        })
    }

    pub fn repack(&self, replacements: &HashMap<usize, Vec<u8>>) -> Result<Vec<u8>> {
        let mut out = self.table.clone();
        let mut cursor = out.len();
        for item in &self.items {
            let block = replacements.get(&item.index).unwrap_or(&item.block);
            ensure!(
                block.get(..4) == Some(b"PACK"),
                "{}: 替换块不是 PACK",
                item.name
            );
            let unpacked_size = read_u32(block, 8)? as usize;
            let declared_packed_size = read_u32(block, 12)? as usize;
            ensure!(
                declared_packed_size == block.len(),
                "{}: PACK 尺寸字段与实际长度不一致",
                item.name
            );

            let entry = PACL_HEADER_SIZE + item.index * PACL_ENTRY_SIZE;
            write_u32(&mut out, entry + 0x10, to_u32(cursor, "PACL 偏移")?)?;
            write_u32(&mut out, entry + 0x14, to_u32(block.len(), "PACK 尺寸")?)?;
            write_u32(&mut out, entry + 0x18, to_u32(unpacked_size, "解压尺寸")?)?;
            out.extend_from_slice(block);
            cursor = out.len();
        }
        Ok(out)
    }
}

pub fn pack_decompress(block: &[u8]) -> Result<Vec<u8>> {
    ensure!(block.get(..4) == Some(b"PACK"), "数据块不是 PACK");
    let unpacked_size = read_u32(block, 8)? as usize;
    let packed_size = read_u32(block, 12)? as usize;
    ensure!(
        packed_size <= block.len(),
        "PACK 尺寸 0x{packed_size:X} 超过数据块 0x{:X}",
        block.len()
    );

    let mut out = Vec::with_capacity(unpacked_size);
    let mut source = PACK_HEADER_SIZE;
    let mut control = 0u8;
    let mut bits = 0u8;
    while source < packed_size && out.len() < unpacked_size {
        if bits == 0 {
            control = *block.get(source).context("PACK 控制字节截断")?;
            source += 1;
            bits = 8;
            continue;
        }
        if control & 0x80 != 0 {
            out.push(*block.get(source).context("PACK 字面量截断")?);
            source += 1;
        } else {
            let pair = block.get(source..source + 2).context("PACK 回溯项截断")?;
            source += 2;
            let word = u16::from_le_bytes(pair.try_into().expect("2-byte slice"));
            let distance = ((word >> 4) as usize) + 1;
            let length = ((word & 0x0F) as usize) + 2;
            let base = out.len() as isize - distance as isize;
            for index in 0..length {
                if out.len() >= unpacked_size {
                    break;
                }
                let from = base + index as isize;
                out.push(if from >= 0 { out[from as usize] } else { 0 });
            }
        }
        control <<= 1;
        bits -= 1;
    }
    ensure!(
        out.len() == unpacked_size,
        "PACK 解压尺寸错误：得到 0x{:X}，预期 0x{unpacked_size:X}",
        out.len()
    );
    Ok(out)
}

pub fn pack_compress_literals(plain: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(PACK_HEADER_SIZE + plain.len() + plain.len() / 8 + 1);
    out.extend_from_slice(b"PACK\0\0\0\0");
    out.extend_from_slice(&to_u32(plain.len(), "解压尺寸")?.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    for chunk in plain.chunks(8) {
        out.push(0xFF);
        out.extend_from_slice(chunk);
    }
    let packed_size = to_u32(out.len(), "PACK 尺寸")?;
    out[12..16].copy_from_slice(&packed_size.to_le_bytes());
    Ok(out)
}

pub fn validate_pack_roundtrip(block: &[u8]) -> Result<()> {
    let plain = pack_decompress(block)?;
    let rebuilt = pack_compress_literals(&plain)?;
    let check = pack_decompress(&rebuilt)?;
    if check != plain {
        bail!("PACK 字面量压缩往返不一致");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{pack_compress_literals, pack_decompress};

    #[test]
    fn literal_pack_roundtrip() {
        let source = (0u8..=255).cycle().take(4097).collect::<Vec<_>>();
        let packed = pack_compress_literals(&source).expect("compress");
        let unpacked = pack_decompress(&packed).expect("decompress");
        assert_eq!(unpacked, source);
    }
}
