use crate::{hex, sha256, Result};
use serde::{Deserialize, Serialize};

pub const SIDE_SIZE: usize = 65_500;
const SIGNATURE: &[u8] = b"\x01*NINTENDO-HVC*";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub side: usize,
    pub index: usize,
    pub number: u8,
    pub id: u8,
    pub name: String,
    pub name_hex: String,
    pub kind: u8,
    pub load_address: u16,
    pub size: usize,
    pub header_offset: usize,
    pub data_offset: usize,
    pub declared: bool,
    pub sha256: String,
}

impl File {
    pub fn output_name(&self) -> String {
        format!("side{}_file{:02}.bin", self.side, self.index)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Side {
    pub index: usize,
    pub info_hex: String,
    pub declared_files: usize,
    pub tail_offset: usize,
    pub tail_size: usize,
    pub tail_sha256: String,
}

#[derive(Debug, Clone)]
pub struct Disk {
    bytes: Vec<u8>,
    pub header_size: usize,
    pub files: Vec<File>,
    pub sides: Vec<Side>,
}

impl Disk {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let header_size = if bytes.starts_with(b"FDS\x1A") { 16 } else { 0 };
        if bytes.len() <= header_size || !(bytes.len() - header_size).is_multiple_of(SIDE_SIZE) {
            return Err("FDS 长度必须是完整的 65500 字节盘面（可有 16 字节 FDS 头）".into());
        }
        let count = (bytes.len() - header_size) / SIDE_SIZE;
        if header_size != 0 && usize::from(bytes[4]) != count {
            return Err("FDS 头中的盘面数量与长度不符".into());
        }
        let mut files = Vec::new();
        let mut sides = Vec::new();
        for side in 0..count {
            let base = header_size + side * SIDE_SIZE;
            let data = &bytes[base..base + SIDE_SIZE];
            if !data.starts_with(SIGNATURE) || data[56] != 2 {
                return Err(format!("盘面 {side}: 磁盘信息／文件数块签名错误").into());
            }
            let declared = usize::from(data[57]);
            let mut pos = 58;
            let mut index = 0;
            while pos < data.len() && data[pos] == 3 {
                let header = data.get(pos..pos + 17).ok_or("FDS 文件头截断")?;
                if header[16] != 4 {
                    return Err(format!("盘面 {side}, ${pos:04X}: 缺少文件数据块 04").into());
                }
                let size = usize::from(u16::from_le_bytes([header[13], header[14]]));
                let kind = header[15];
                let load = u16::from_le_bytes([header[11], header[12]]);
                let bound = match kind {
                    0 => 0x10000,
                    1 => 0x2000,
                    2 => 0x4000,
                    _ => return Err(format!("未知 FDS 文件类型 {kind}").into()),
                };
                if usize::from(load) + size > bound {
                    return Err(format!("盘面 {side} 文件 {index}: 装载范围越界").into());
                }
                let payload = data
                    .get(pos + 17..pos + 17 + size)
                    .ok_or("FDS 文件跨越盘面末端")?;
                let name_raw = &header[3..11];
                let name = String::from_utf8_lossy(name_raw)
                    .trim_end_matches(['\0', ' '])
                    .to_owned();
                files.push(File {
                    side,
                    index,
                    number: header[1],
                    id: header[2],
                    name,
                    name_hex: hex(name_raw),
                    kind,
                    load_address: load,
                    size,
                    header_offset: base + pos,
                    data_offset: base + pos + 17,
                    declared: index < declared,
                    sha256: sha256(payload),
                });
                index += 1;
                pos += 17 + size;
            }
            if index < declared {
                return Err(
                    format!("盘面 {side}: 声明 {declared} 个文件，只找到 {index} 个").into(),
                );
            }
            sides.push(Side {
                index: side,
                info_hex: hex(&data[..56]),
                declared_files: declared,
                tail_offset: base + pos,
                tail_size: SIDE_SIZE - pos,
                tail_sha256: sha256(&data[pos..]),
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
            header_size,
            files,
            sides,
        })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn payload(&self, file: &File) -> &[u8] {
        &self.bytes[file.data_offset..file.data_offset + file.size]
    }

    pub fn file(&self, name: &str) -> Result<&File> {
        let found = self
            .files
            .iter()
            .filter(|f| f.name == name)
            .collect::<Vec<_>>();
        if found.len() != 1 {
            return Err(format!("需要唯一的 FDS 文件 {name}，找到 {} 个", found.len()).into());
        }
        Ok(found[0])
    }

    pub fn rebuild(
        &self,
        replacements: &std::collections::BTreeMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>> {
        for name in replacements.keys() {
            self.file(name)?;
        }
        let mut result = self.bytes[..self.header_size].to_vec();
        for side in &self.sides {
            let base = self.header_size + side.index * SIDE_SIZE;
            let mut rebuilt = self.bytes[base..base + 58].to_vec();
            for file in self.files.iter().filter(|f| f.side == side.index) {
                let payload = replacements
                    .get(&file.name)
                    .map_or(self.payload(file), Vec::as_slice);
                let length =
                    u16::try_from(payload.len()).map_err(|_| "FDS 单文件超过 65535 字节")?;
                let bound = match file.kind {
                    0 => 0x10000,
                    1 => 0x2000,
                    _ => 0x4000,
                };
                if usize::from(file.load_address) + payload.len() > bound {
                    return Err(format!("{} 的装载范围越界", file.name).into());
                }
                let mut header = self.bytes[file.header_offset..file.data_offset].to_vec();
                header[13..15].copy_from_slice(&length.to_le_bytes());
                rebuilt.extend_from_slice(&header);
                rebuilt.extend_from_slice(payload);
            }
            let old_used = side.tail_offset - base;
            let tail = &self.bytes[side.tail_offset..base + SIDE_SIZE];
            if rebuilt.len() == old_used {
                rebuilt.extend_from_slice(tail);
            } else if tail.iter().all(|b| *b == 0) && rebuilt.len() <= SIDE_SIZE {
                rebuilt.resize(SIDE_SIZE, 0);
            } else {
                return Err(format!("盘面 {} 容量不足，或增长将覆盖不透明盘尾", side.index).into());
            }
            if rebuilt.len() != SIDE_SIZE {
                return Err("FDS 重建盘面长度错误".into());
            }
            result.extend_from_slice(&rebuilt);
        }
        Self::parse(&result)?;
        Ok(result)
    }
}
