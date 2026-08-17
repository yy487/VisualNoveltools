use crate::{Result, fail};
use std::collections::{HashMap, HashSet};

const VERSION_9795: u16 = 0x9795;

#[derive(Clone, Debug)]
pub struct Opcode {
    pub opcode: u8,
    pub content: Vec<u8>,
    pub original_offset: u32,
    pub original_total_length: usize,
}

#[derive(Clone, Debug)]
pub struct IsfScript {
    pub minyan_prefix: bool,
    pub head_len: u32,
    pub version_info: [u8; 4],
    pub header_offsets: Vec<u32>,
    pub opcodes: Vec<Opcode>,
    original_body_len: u32,
}

#[derive(Clone, Debug)]
pub struct Replacement {
    pub start: usize,
    pub end: usize,
    pub data: Vec<u8>,
}

impl IsfScript {
    pub fn parse(on_disk: &[u8]) -> Result<Self> {
        let (minyan_prefix, encrypted) = if let Some(rest) = on_disk.strip_prefix(b"MINYAN") {
            (true, rest)
        } else {
            (false, on_disk)
        };
        if encrypted.len() < 8 {
            return fail("ISF is smaller than its fixed eight-byte header");
        }
        let mut decoded = encrypted.to_vec();
        let version = u16::from_le_bytes([decoded[4], decoded[5]]);
        if version != VERSION_9795 {
            return fail(format!(
                "unsupported ISF version 0x{version:04X}; Lien requires 0x9795"
            ));
        }
        for byte in &mut decoded[8..] {
            *byte = byte.rotate_right(2);
        }

        let head_len = u32::from_le_bytes(decoded[0..4].try_into().expect("fixed slice"));
        let head_len_usize = head_len as usize;
        if head_len_usize < 8 || head_len_usize > decoded.len() {
            return fail(format!(
                "invalid ISF header length 0x{head_len:X} for file size 0x{:X}",
                decoded.len()
            ));
        }
        if !(head_len_usize - 8).is_multiple_of(4) {
            return fail(format!(
                "ISF header offset area has non-u32 remainder: head_len=0x{head_len:X}"
            ));
        }
        let version_info = decoded[4..8].try_into().expect("fixed slice");
        let mut header_offsets = Vec::with_capacity((head_len_usize - 8) / 4);
        for chunk in decoded[8..head_len_usize].chunks_exact(4) {
            header_offsets.push(u32::from_le_bytes(chunk.try_into().expect("fixed slice")));
        }

        let body = &decoded[head_len_usize..];
        let mut position = 0usize;
        let mut opcodes = Vec::new();
        while position < body.len() {
            let start = position;
            if position + 2 > body.len() {
                return fail(format!("truncated ISF opcode at body offset 0x{start:X}"));
            }
            let opcode = body[position];
            let length_byte = body[position + 1];
            position += 2;
            let (total_length, header_length) = if length_byte < 0x80 {
                (length_byte as usize, 2usize)
            } else {
                if position >= body.len() {
                    return fail(format!(
                        "truncated extended ISF length at body offset 0x{start:X}"
                    ));
                }
                let low = body[position] as usize;
                position += 1;
                ((length_byte as usize - 0x80) * 0x100 + low, 3usize)
            };
            if total_length < header_length {
                return fail(format!(
                    "invalid ISF opcode length {total_length} at body offset 0x{start:X}"
                ));
            }
            let content_length = total_length - header_length;
            let end = position
                .checked_add(content_length)
                .ok_or("ISF opcode size overflow")?;
            if end > body.len() {
                return fail(format!(
                    "ISF opcode at body offset 0x{start:X} ends at 0x{end:X}, beyond body size 0x{:X}",
                    body.len()
                ));
            }
            opcodes.push(Opcode {
                opcode,
                content: body[position..end].to_vec(),
                original_offset: u32::try_from(start).map_err(|_| "ISF body offset exceeds u32")?,
                original_total_length: total_length,
            });
            position = end;
        }
        let original_body_len = u32::try_from(body.len()).map_err(|_| "ISF body exceeds u32")?;

        let mut boundaries: HashSet<u32> = opcodes.iter().map(|op| op.original_offset).collect();
        boundaries.insert(original_body_len);
        for (index, offset) in header_offsets.iter().enumerate() {
            if !boundaries.contains(offset) {
                return fail(format!(
                    "ISF header offset #{index} (0x{offset:X}) does not target an opcode boundary"
                ));
            }
        }

        Ok(Self {
            minyan_prefix,
            head_len,
            version_info,
            header_offsets,
            opcodes,
            original_body_len,
        })
    }

    pub fn rebuild(&self) -> Result<Vec<u8>> {
        let mut old_target_to_index = HashMap::with_capacity(self.opcodes.len() + 1);
        for (index, opcode) in self.opcodes.iter().enumerate() {
            if old_target_to_index
                .insert(opcode.original_offset, index)
                .is_some()
            {
                return fail(format!(
                    "duplicate original ISF opcode offset 0x{:X}",
                    opcode.original_offset
                ));
            }
        }
        old_target_to_index.insert(self.original_body_len, self.opcodes.len());

        let mut body = Vec::new();
        let mut new_offsets = Vec::with_capacity(self.opcodes.len() + 1);
        for opcode in &self.opcodes {
            new_offsets
                .push(u32::try_from(body.len()).map_err(|_| "rebuilt ISF body exceeds u32")?);
            serialize_opcode(opcode, &mut body)?;
        }
        new_offsets.push(u32::try_from(body.len()).map_err(|_| "rebuilt ISF body exceeds u32")?);

        let expected_head_len = 8usize
            .checked_add(self.header_offsets.len() * 4)
            .ok_or("ISF header size overflow")?;
        if expected_head_len != self.head_len as usize {
            return fail(format!(
                "ISF header model mismatch: calculated 0x{expected_head_len:X}, stored 0x{:X}",
                self.head_len
            ));
        }
        let mut decoded = Vec::with_capacity(expected_head_len + body.len());
        decoded.extend_from_slice(&self.head_len.to_le_bytes());
        decoded.extend_from_slice(&self.version_info);
        for (table_index, old_offset) in self.header_offsets.iter().enumerate() {
            let opcode_index = old_target_to_index.get(old_offset).ok_or_else(|| {
                format!(
                    "cannot relocate ISF header offset #{table_index}: unknown old target 0x{old_offset:X}"
                )
            })?;
            decoded.extend_from_slice(&new_offsets[*opcode_index].to_le_bytes());
        }
        decoded.extend_from_slice(&body);
        for byte in &mut decoded[8..] {
            *byte = byte.rotate_left(2);
        }
        if self.minyan_prefix {
            let mut output = Vec::with_capacity(decoded.len() + 6);
            output.extend_from_slice(b"MINYAN");
            output.extend_from_slice(&decoded);
            Ok(output)
        } else {
            Ok(decoded)
        }
    }

    pub fn apply_replacements(
        &mut self,
        opcode_index: usize,
        mut replacements: Vec<Replacement>,
    ) -> Result<()> {
        let opcode = self
            .opcodes
            .get_mut(opcode_index)
            .ok_or_else(|| format!("ISF opcode index {opcode_index} is outside the script"))?;
        replacements.sort_by_key(|replacement| replacement.start);
        let mut previous_end = 0usize;
        for replacement in &replacements {
            if replacement.start > replacement.end || replacement.end > opcode.content.len() {
                return fail(format!(
                    "invalid replacement range 0x{:X}..0x{:X} for opcode #{opcode_index} content size 0x{:X}",
                    replacement.start,
                    replacement.end,
                    opcode.content.len()
                ));
            }
            if replacement.start < previous_end {
                return fail(format!(
                    "overlapping replacements in opcode #{opcode_index} at content offset 0x{:X}",
                    replacement.start
                ));
            }
            previous_end = replacement.end;
        }
        for replacement in replacements.into_iter().rev() {
            opcode
                .content
                .splice(replacement.start..replacement.end, replacement.data);
        }
        Ok(())
    }
}

fn serialize_opcode(opcode: &Opcode, output: &mut Vec<u8>) -> Result<()> {
    let short_total = opcode
        .content
        .len()
        .checked_add(2)
        .ok_or("ISF opcode size overflow")?;
    output.push(opcode.opcode);
    if short_total < 0x80 {
        output.push(short_total as u8);
    } else {
        let extended_total = opcode
            .content
            .len()
            .checked_add(3)
            .ok_or("ISF opcode size overflow")?;
        let high = extended_total / 0x100;
        if high > 0x7F {
            return fail(format!(
                "rebuilt opcode 0x{:02X} is too large: {extended_total} bytes",
                opcode.opcode
            ));
        }
        output.push(0x80 + high as u8);
        output.push((extended_total & 0xFF) as u8);
    }
    output.extend_from_slice(&opcode.content);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encrypted_sample() -> Vec<u8> {
        let mut decoded = Vec::new();
        decoded.extend_from_slice(&12u32.to_le_bytes());
        decoded.extend_from_slice(&[0x95, 0x97, 0x00, 0x00]);
        decoded.extend_from_slice(&0u32.to_le_bytes());
        decoded.extend_from_slice(&[0x2B, 0x06, 0x00, 0xFF, 0x15, 0x00]);
        for byte in &mut decoded[8..] {
            *byte = byte.rotate_left(2);
        }
        decoded
    }

    #[test]
    fn isf_roundtrip_is_exact() {
        let bytes = encrypted_sample();
        let parsed = IsfScript::parse(&bytes).unwrap();
        assert_eq!(parsed.rebuild().unwrap(), bytes);
    }

    #[test]
    fn relocation_updates_header_offsets() {
        let bytes = encrypted_sample();
        let mut parsed = IsfScript::parse(&bytes).unwrap();
        parsed
            .apply_replacements(
                0,
                vec![Replacement {
                    start: 3,
                    end: 3,
                    data: vec![0x82, 0xA0],
                }],
            )
            .unwrap();
        let rebuilt = parsed.rebuild().unwrap();
        let reparsed = IsfScript::parse(&rebuilt).unwrap();
        assert_eq!(reparsed.header_offsets, vec![0]);
        assert_eq!(reparsed.opcodes[0].content.len(), 6);
    }
}
