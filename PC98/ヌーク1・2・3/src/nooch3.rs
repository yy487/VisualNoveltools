use crate::codec::{lzss3_compress, lzss3_decode, read_u16, read_u32};
use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ScriptField {
    pub index: usize,
    pub category: &'static str,
    pub prefix_start: usize,
    pub data_start: usize,
    pub data_end: usize,
    pub length_width: usize,
}

#[derive(Debug, Clone)]
pub struct SceneRecord {
    pub id: u16,
    pub raw: Vec<u8>,
    pub unpacked: Vec<u8>,
    pub scripts: Vec<ScriptField>,
}

#[derive(Debug, Clone)]
pub struct SceneArchive {
    pub records: Vec<SceneRecord>,
}

impl SceneArchive {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let count = read_u16(data, 0)? as usize;
        let ids_start = 2usize;
        let offsets_start = ids_start
            .checked_add(count * 2)
            .context("三代场景 ID 表溢出")?;
        let base = offsets_start
            .checked_add(count * 4)
            .context("三代场景偏移表溢出")?;
        if base > data.len() {
            bail!("三代 SEEN_A.TXT 表头越界");
        }
        let mut records = Vec::with_capacity(count);
        for index in 0..count {
            let id = read_u16(data, ids_start + index * 2)?;
            let offset = read_u32(data, offsets_start + index * 4)? as usize;
            let next = if index + 1 < count {
                read_u32(data, offsets_start + (index + 1) * 4)? as usize
            } else {
                data.len() - base
            };
            let raw = data
                .get(base + offset..base + next)
                .with_context(|| format!("三代场景记录 {index} 越界"))?
                .to_vec();
            let packed_size = read_u16(&raw, 0)? as usize;
            let expected_size = read_u16(&raw, 2)? as usize;
            let packed = raw
                .get(4..4 + packed_size)
                .with_context(|| format!("三代场景 {id} 压缩流越界"))?;
            let unpacked = lzss3_decode(packed)?;
            if unpacked.len() != expected_size {
                bail!(
                    "三代场景 {id} 解压长度 {} != 声明 {expected_size}",
                    unpacked.len()
                );
            }
            let scripts =
                parse_script_fields(&unpacked).with_context(|| format!("解析三代场景 {id}"))?;
            records.push(SceneRecord {
                id,
                raw,
                unpacked,
                scripts,
            });
        }
        Ok(Self { records })
    }

    pub fn rebuild(&self, changes: &BTreeMap<usize, BTreeMap<usize, Vec<u8>>>) -> Result<Vec<u8>> {
        if self.records.len() > u16::MAX as usize {
            bail!("三代场景记录数超过 u16");
        }
        let mut record_bytes = Vec::with_capacity(self.records.len());
        for (record_index, record) in self.records.iter().enumerate() {
            let Some(script_changes) = changes.get(&record_index) else {
                record_bytes.push(record.raw.clone());
                continue;
            };
            let mut unpacked = record.unpacked.clone();
            let mut edits = Vec::new();
            for (&script_index, replacement) in script_changes {
                let field = record
                    .scripts
                    .get(script_index)
                    .with_context(|| format!("场景 {} 不存在脚本 {script_index}", record.id))?;
                let length = replacement.len();
                let mut encoded = Vec::new();
                match field.length_width {
                    1 => {
                        if length > u8::MAX as usize {
                            bail!("场景 {} 的 {} 脚本超过 255 字节", record.id, field.category);
                        }
                        encoded.push(length as u8);
                    }
                    2 => {
                        if length > u16::MAX as usize {
                            bail!(
                                "场景 {} 的 {} 脚本超过 65535 字节",
                                record.id,
                                field.category
                            );
                        }
                        encoded.extend_from_slice(&(length as u16).to_le_bytes());
                    }
                    _ => unreachable!(),
                }
                encoded.extend_from_slice(replacement);
                edits.push((field.prefix_start, field.data_end, encoded));
            }
            edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
            for (start, end, replacement) in edits {
                unpacked.splice(start..end, replacement);
            }
            let packed = lzss3_compress(&unpacked)?;
            if packed.len() > u16::MAX as usize {
                bail!("场景 {} 的重建压缩流超过 65535 字节", record.id);
            }
            let mut raw = Vec::with_capacity(4 + packed.len());
            raw.extend_from_slice(&(packed.len() as u16).to_le_bytes());
            raw.extend_from_slice(&(unpacked.len() as u16).to_le_bytes());
            raw.extend_from_slice(&packed);
            record_bytes.push(raw);
        }
        let count = self.records.len();
        let mut output = Vec::new();
        output.extend_from_slice(&(count as u16).to_le_bytes());
        for record in &self.records {
            output.extend_from_slice(&record.id.to_le_bytes());
        }
        let mut offset = 0usize;
        for raw in &record_bytes {
            if offset > u32::MAX as usize {
                bail!("三代场景归档偏移超过 u32");
            }
            output.extend_from_slice(&(offset as u32).to_le_bytes());
            offset += raw.len();
        }
        for raw in record_bytes {
            output.extend_from_slice(&raw);
        }
        Ok(output)
    }
}

fn parse_script_fields(data: &[u8]) -> Result<Vec<ScriptField>> {
    let mut parser = Parser {
        data,
        pos: 0,
        fields: Vec::new(),
    };
    for _ in 0..3 {
        parser.script(2, "prelude")?;
    }
    parser.skip(25)?;
    parser.skip(1)?;
    let state_count = parser.byte()? as usize;
    parser.skip(8)?;
    let mut group_counts = Vec::with_capacity(state_count);
    for _ in 0..state_count {
        group_counts.push(parser.byte()? as usize);
        parser.skip(8)?;
    }
    let mut option_counts = Vec::with_capacity(state_count);
    for &group_count in &group_counts {
        let mut state = Vec::with_capacity(group_count);
        for _ in 0..group_count {
            let option_count = parser.byte()? as usize;
            state.push(option_count);
            for _ in 0..option_count {
                let descriptor_count = parser.byte()? as usize;
                parser.skip(descriptor_count.checked_mul(3).context("描述符长度溢出")?)?;
            }
        }
        option_counts.push(state);
    }
    let aux_count = parser.byte()? as usize;
    for _ in 0..aux_count {
        parser.script(2, "aux")?;
    }
    for _ in 0..state_count {
        parser.script(1, "state")?;
    }
    for &group_count in &group_counts {
        for _ in 0..group_count {
            parser.script(1, "group")?;
        }
    }
    for state in option_counts {
        for option_count in state {
            for _ in 0..option_count {
                parser.script(2, "option")?;
            }
        }
    }
    if parser.pos != data.len() {
        bail!(
            "脚本结构结束于 0x{:X}，记录实际长度 0x{:X}",
            parser.pos,
            data.len()
        );
    }
    Ok(parser.fields)
}

struct Parser<'a> {
    data: &'a [u8],
    pos: usize,
    fields: Vec<ScriptField>,
}

impl Parser<'_> {
    fn byte(&mut self) -> Result<u8> {
        let value = *self.data.get(self.pos).context("读取 u8 越界")?;
        self.pos += 1;
        Ok(value)
    }

    fn skip(&mut self, count: usize) -> Result<()> {
        self.pos = self.pos.checked_add(count).context("结构偏移溢出")?;
        if self.pos > self.data.len() {
            bail!("结构字段越界");
        }
        Ok(())
    }

    fn script(&mut self, width: usize, category: &'static str) -> Result<()> {
        let prefix_start = self.pos;
        let length = if width == 1 {
            self.byte()? as usize
        } else {
            let value = read_u16(self.data, self.pos)? as usize;
            self.pos += 2;
            value
        };
        let data_start = self.pos;
        self.skip(length)?;
        self.fields.push(ScriptField {
            index: self.fields.len(),
            category,
            prefix_start,
            data_start,
            data_end: self.pos,
            length_width: width,
        });
        Ok(())
    }
}
