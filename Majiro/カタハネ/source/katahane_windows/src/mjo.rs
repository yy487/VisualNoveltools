const HEADER_SIZE: usize = 0x1c;
const SYMBOL_SIZE: usize = 8;
const V1_MAGIC: &[u8; 16] = b"MajiroObjV1.000\0";
const X1_MAGIC: &[u8; 16] = b"MajiroObjX1.000\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MjoVersion {
    V1,
    X1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MjoDocument {
    pub version: MjoVersion,
    pub entrypoint: u32,
    pub mark_size: u32,
    pub symbols: Vec<(u32, u32)>,
    pub payload_size: usize,
    payload_start: usize,
    payload: Vec<u8>,
}

impl MjoDocument {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < HEADER_SIZE {
            return Err("文件短于 MajiroObj 头部".into());
        }
        let version = if &bytes[..16] == V1_MAGIC {
            MjoVersion::V1
        } else if &bytes[..16] == X1_MAGIC {
            MjoVersion::X1
        } else {
            return Err("签名不匹配：需要 MajiroObjV1.000 或 MajiroObjX1.000".into());
        };
        let entrypoint = read_u32(bytes, 0x10)?;
        let mark_size = read_u32(bytes, 0x14)?;
        let symbol_count = read_u32(bytes, 0x18)? as usize;
        let symbols_end = HEADER_SIZE
            .checked_add(
                symbol_count
                    .checked_mul(SYMBOL_SIZE)
                    .ok_or("符号表长度溢出")?,
            )
            .ok_or("符号表末尾溢出")?;
        let payload_size = read_u32(bytes, symbols_end)? as usize;
        let payload_start = symbols_end.checked_add(4).ok_or("负载偏移溢出")?;
        let payload_end = payload_start
            .checked_add(payload_size)
            .ok_or("负载末尾溢出")?;
        if payload_end > bytes.len() {
            return Err(format!(
                "MajiroObj 声明负载长度为 {payload_size}，但文件在偏移 0x{payload_start:X} 后不足"
            ));
        }
        if entrypoint != u32::MAX && entrypoint as usize > payload_size {
            return Err("MajiroObj 入口偏移超出负载范围".into());
        }

        let mut symbols = Vec::with_capacity(symbol_count);
        for i in 0..symbol_count {
            let offset = HEADER_SIZE + i * SYMBOL_SIZE;
            let hash = read_u32(bytes, offset)?;
            let symbol_offset = read_u32(bytes, offset + 4)?;
            if symbol_offset as usize > payload_size {
                return Err(format!("MajiroObj 符号表第 {} 项偏移超出负载范围", i + 1));
            }
            symbols.push((hash, symbol_offset));
        }
        let mut payload = bytes[payload_start..payload_end].to_vec();
        if version == MjoVersion::X1 {
            xor_crc32_table(&mut payload);
        }
        Ok(Self {
            version,
            entrypoint,
            mark_size,
            symbols,
            payload_size,
            payload_start,
            payload,
        })
    }

    /// Return the same object with its X1 payload decoded and its magic changed to V1.
    /// The header, symbol table, declared length, and any trailing bytes are preserved.
    pub fn as_v1_bytes(&self, original: &[u8]) -> Result<Vec<u8>, String> {
        if original.len() < self.payload_start + self.payload_size {
            return Err("原始对象在准备完成后变短".into());
        }
        let mut decoded = original.to_vec();
        decoded[..16].copy_from_slice(V1_MAGIC);
        let end = self.payload_start + self.payload_size;
        decoded[self.payload_start..end].copy_from_slice(&self.payload);
        Ok(decoded)
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Rebuild this object with a relocated V1 payload, retaining its original
    /// version, header metadata, symbol hashes, and any bytes after the payload.
    pub fn rebuild_with_payload(
        &self,
        original: &[u8],
        payload: &[u8],
        symbols: &[(u32, u32)],
        entrypoint: u32,
    ) -> Result<Vec<u8>, String> {
        let original_payload_end = self
            .payload_start
            .checked_add(self.payload_size)
            .ok_or("原始负载末尾溢出")?;
        if original.len() < original_payload_end {
            return Err("原始对象在准备完成后变短".into());
        }
        if payload.len() > u32::MAX as usize {
            return Err("重建后的 MJO 负载超过 32 位长度范围".into());
        }
        if symbols.len() != self.symbols.len() {
            return Err("重建后的 MJO 符号数量发生变化".into());
        }
        for (index, ((old_hash, _), (new_hash, offset))) in
            self.symbols.iter().zip(symbols).enumerate()
        {
            if old_hash != new_hash {
                return Err(format!("重建后的第 {} 个符号哈希发生变化", index + 1));
            }
            if *offset as usize >= payload.len() {
                return Err(format!("重建后的第 {} 个符号偏移越界", index + 1));
            }
        }
        if entrypoint != u32::MAX && entrypoint as usize >= payload.len() {
            return Err("重建后的 MJO 入口偏移越界".into());
        }

        let tail = &original[original_payload_end..];
        let final_len = self
            .payload_start
            .checked_add(payload.len())
            .and_then(|length| length.checked_add(tail.len()))
            .ok_or("重建后的 MJO 文件长度溢出")?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(final_len)
            .map_err(|error| format!("无法为重建后的 MJO 分配空间: {error}"))?;
        output.extend_from_slice(&original[..self.payload_start]);
        output[..16].copy_from_slice(match self.version {
            MjoVersion::V1 => V1_MAGIC,
            MjoVersion::X1 => X1_MAGIC,
        });
        output[0x10..0x14].copy_from_slice(&entrypoint.to_le_bytes());
        for (index, (_, offset)) in symbols.iter().enumerate() {
            let field = HEADER_SIZE + index * SYMBOL_SIZE + 4;
            output[field..field + 4].copy_from_slice(&offset.to_le_bytes());
        }
        let payload_size_field = self.payload_start - 4;
        output[payload_size_field..self.payload_start]
            .copy_from_slice(&(payload.len() as u32).to_le_bytes());
        let mut stored_payload = payload.to_vec();
        if self.version == MjoVersion::X1 {
            xor_crc32_table(&mut stored_payload);
        }
        output.extend_from_slice(&stored_payload);
        output.extend_from_slice(tail);
        if output.len() != final_len {
            return Err("重建后的 MJO 文件长度与预计算值不一致".into());
        }
        Ok(output)
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let end = offset.checked_add(4).ok_or("字段偏移溢出")?;
    let field = bytes
        .get(offset..end)
        .ok_or_else(|| format!("MajiroObj 字段越界: 0x{offset:X}"))?;
    Ok(u32::from_le_bytes(
        field.try_into().expect("four-byte slice"),
    ))
}

fn xor_crc32_table(payload: &mut [u8]) {
    let mut key = [0u8; 1024];
    for index in 0..256 {
        let mut value = index as u32;
        for _ in 0..8 {
            value = if value & 1 != 0 {
                (value >> 1) ^ 0xedb8_8320
            } else {
                value >> 1
            };
        }
        key[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= key[index % key.len()];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_x1(payload: &[u8], trailing: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; HEADER_SIZE + 8 + 4];
        bytes[..16].copy_from_slice(X1_MAGIC);
        bytes[0x10..0x14].copy_from_slice(&0u32.to_le_bytes());
        bytes[0x14..0x18].copy_from_slice(&12u32.to_le_bytes());
        bytes[0x18..0x1c].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x1c..0x20].copy_from_slice(&0x1234_5678u32.to_le_bytes());
        bytes[0x20..0x24].copy_from_slice(&0u32.to_le_bytes());
        bytes[0x24..0x28].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        let mut encrypted = payload.to_vec();
        xor_crc32_table(&mut encrypted);
        bytes.extend_from_slice(&encrypted);
        bytes.extend_from_slice(trailing);
        bytes
    }

    #[test]
    fn decodes_x1_payload_and_preserves_other_bytes() {
        let original = make_x1(b"CP932 text\0 with opaque bytes", b"tail");
        let parsed = MjoDocument::parse(&original).unwrap();
        assert_eq!(parsed.version, MjoVersion::X1);
        assert_eq!(parsed.entrypoint, 0);
        assert_eq!(parsed.mark_size, 12);
        assert_eq!(parsed.symbols, [(0x1234_5678, 0)]);
        assert_eq!(parsed.payload, b"CP932 text\0 with opaque bytes");
        let decoded = parsed.as_v1_bytes(&original).unwrap();
        assert_eq!(&decoded[..16], V1_MAGIC);
        assert_eq!(
            &decoded[parsed.payload_start..parsed.payload_start + parsed.payload_size],
            b"CP932 text\0 with opaque bytes"
        );
        assert_eq!(&decoded[decoded.len() - 4..], b"tail");
    }

    #[test]
    fn accepts_v1_payload_without_transform() {
        let mut bytes = vec![0; HEADER_SIZE + 4];
        bytes[..16].copy_from_slice(V1_MAGIC);
        bytes[0x1c..0x20].copy_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(b"plain");
        let parsed = MjoDocument::parse(&bytes).unwrap();
        assert_eq!(parsed.version, MjoVersion::V1);
        assert_eq!(parsed.payload, b"plai");
        assert_eq!(parsed.as_v1_bytes(&bytes).unwrap(), bytes);
    }

    #[test]
    fn rejects_truncated_symbol_table_or_payload() {
        let mut short_table = vec![0; HEADER_SIZE];
        short_table[..16].copy_from_slice(V1_MAGIC);
        short_table[0x18..0x1c].copy_from_slice(&1u32.to_le_bytes());
        assert!(MjoDocument::parse(&short_table).is_err());

        let mut short_payload = vec![0; HEADER_SIZE + 4];
        short_payload[..16].copy_from_slice(V1_MAGIC);
        short_payload[0x1c..0x20].copy_from_slice(&10u32.to_le_bytes());
        assert!(MjoDocument::parse(&short_payload).is_err());
    }

    #[test]
    fn rejects_entrypoint_and_symbol_offsets_past_payload() {
        let mut bytes = make_x1(b"abc", b"");
        bytes[0x10..0x14].copy_from_slice(&4u32.to_le_bytes());
        assert!(MjoDocument::parse(&bytes).is_err());

        let mut bytes = make_x1(b"abc", b"");
        bytes[0x20..0x24].copy_from_slice(&4u32.to_le_bytes());
        assert!(MjoDocument::parse(&bytes).is_err());
    }
}
