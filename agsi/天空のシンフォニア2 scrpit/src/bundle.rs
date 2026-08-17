use crate::encoding::decode_cp932;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

pub type BundleError = String;

#[derive(Debug, Clone)]
pub struct CStrEntry {
    pub id: usize,
    pub pool_offset: usize,
    pub size: usize,
    pub raw: Vec<u8>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub file_id: usize,
    pub file: String,
    pub line: u32,
    pub dbg_record: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub offset: usize,
    pub opcode: u8,
    pub operand: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ApiEntry {
    pub index: usize,
    pub name: String,
    pub address: usize,
    pub argc: usize,
    pub unknown: u32,
}

#[derive(Debug, Clone)]
pub struct TalkInfoRecord {
    pub talk_info_id: u32,
    pub name_cstr_id: usize,
    pub name: String,
    pub trailing_ints: Vec<u32>,
    pub call_offset: usize,
    pub source: SourceInfo,
}

#[derive(Debug, Clone)]
pub struct Bundle {
    pub root: PathBuf,
    pub manifest: Value,
    pub code: Vec<u8>,
    pub instructions: Vec<Instruction>,
    pub instruction_index: HashMap<usize, usize>,
    pub cstr: Vec<CStrEntry>,
    pub ftbl0: Vec<ApiEntry>,
    pub ftbl1: Vec<ApiEntry>,
    pub api_by_address: HashMap<usize, ApiEntry>,
    pub source_files: Vec<String>,
    pub line_records: Vec<(usize, usize, u32)>,
    pub function_entries: BTreeMap<usize, u32>,
    pub function_ranges: BTreeMap<usize, (usize, usize)>,
}

fn read_file(root: &Path, name: &str) -> Result<Vec<u8>, BundleError> {
    let path = root.join(name);
    fs::read(&path).map_err(|e| format!("读取 {} 失败: {}", path.display(), e))
}

fn read_u32(data: &[u8], offset: usize, context: &str) -> Result<u32, BundleError> {
    if offset.checked_add(4).is_none() || offset + 4 > data.len() {
        return Err(format!("{} 在 0x{:x} 截断", context, offset));
    }
    Ok(u32::from_le_bytes(
        data[offset..offset + 4].try_into().unwrap(),
    ))
}

fn swap_nibbles(data: &[u8]) -> Vec<u8> {
    data.iter()
        .map(|value| (value >> 4) | ((value & 0x0f) << 4))
        .collect()
}

fn parse_cstr(data: &[u8], count: usize) -> Result<Vec<CStrEntry>, BundleError> {
    let table_size = count
        .checked_mul(8)
        .ok_or_else(|| "CSTR 表大小溢出".to_string())?;
    if data.len() < table_size {
        return Err(format!("CSTR 表超出文件: {} < {}", data.len(), table_size));
    }
    let pool = swap_nibbles(&data[table_size..]);
    let mut entries = Vec::with_capacity(count);
    let mut expected_offset = 0usize;
    for id in 0..count {
        let offset = read_u32(data, id * 8, &format!("CSTR[{}] offset", id))? as usize;
        let size = read_u32(data, id * 8 + 4, &format!("CSTR[{}] size", id))? as usize;
        let end = offset
            .checked_add(size)
            .ok_or_else(|| format!("CSTR[{}] 范围溢出", id))?;
        if offset != expected_offset {
            return Err(format!(
                "CSTR[{}] 不连续: 0x{:x} != 0x{:x}",
                id, offset, expected_offset
            ));
        }
        if end > pool.len() {
            return Err(format!(
                "CSTR[{}] 越界: 0x{:x} > 0x{:x}",
                id,
                end,
                pool.len()
            ));
        }
        let raw = pool[offset..end].to_vec();
        if !raw.ends_with(&[0]) || raw[..raw.len().saturating_sub(1)].contains(&0) {
            return Err(format!("CSTR[{}] NUL 结构异常", id));
        }
        let body = &raw[..raw.len() - 1];
        let text = decode_cp932(body, &format!("CSTR[{}]", id))?;
        entries.push(CStrEntry {
            id,
            pool_offset: offset,
            size,
            raw,
            text,
        });
        expected_offset = end;
    }
    if expected_offset != pool.len() {
        return Err(format!(
            "CSTR 池未完全覆盖: 0x{:x} != 0x{:x}",
            expected_offset,
            pool.len()
        ));
    }
    Ok(entries)
}

fn parse_ftbl(data: &[u8], context: &str) -> Result<Vec<ApiEntry>, BundleError> {
    let mut entries = Vec::new();
    let mut offset = 0usize;
    while offset < data.len() {
        let name_size = read_u32(data, offset, &format!("{} name size", context))? as usize;
        offset += 4;
        let end = offset
            .checked_add(name_size)
            .ok_or_else(|| format!("{} name size 溢出", context))?;
        if end.checked_add(12).is_none() || end + 12 > data.len() {
            return Err(format!("{} record {} 截断", context, entries.len()));
        }
        let raw_name = &data[offset..end];
        offset = end;
        if !raw_name.ends_with(&[0]) {
            return Err(format!("{} record {} 缺少 NUL", context, entries.len()));
        }
        let name = decode_cp932(
            &raw_name[..raw_name.len() - 1],
            &format!("{} name", context),
        )?;
        let unknown = read_u32(data, offset, &format!("{} unknown", context))?;
        let address = read_u32(data, offset + 4, &format!("{} address", context))? as usize;
        let argc = read_u32(data, offset + 8, &format!("{} argc", context))? as usize;
        offset += 12;
        entries.push(ApiEntry {
            index: entries.len(),
            name,
            address,
            argc,
            unknown,
        });
    }
    Ok(entries)
}

fn parse_debug_files(data: &[u8]) -> Result<Vec<String>, BundleError> {
    let count = read_u32(data, 0, "DBG_0 count")? as usize;
    let mut offset = 4usize;
    let mut files = Vec::with_capacity(count);
    for index in 0..count {
        let size = read_u32(data, offset, &format!("DBG_0[{}] size", index))? as usize;
        offset += 4;
        let end = offset
            .checked_add(size)
            .ok_or_else(|| format!("DBG_0[{}] size 溢出", index))?;
        if end > data.len() {
            return Err(format!("DBG_0[{}] 越界", index));
        }
        let raw = swap_nibbles(&data[offset..end]);
        offset = end;
        if !raw.ends_with(&[0]) {
            return Err(format!("DBG_0[{}] 缺少 NUL", index));
        }
        let decoded = decode_cp932(&raw[..raw.len() - 1], &format!("DBG_0[{}]", index))?;
        let normalized = decoded.replace('/', "\\");
        let file = normalized
            .rsplit('\\')
            .next()
            .unwrap_or(&normalized)
            .to_string();
        files.push(file);
    }
    if offset != data.len() {
        return Err("DBG_0 有尾随数据".to_string());
    }
    Ok(files)
}

fn parse_debug_lines(
    data: &[u8],
    file_count: usize,
) -> Result<Vec<(usize, usize, u32)>, BundleError> {
    let count = read_u32(data, 0, "DBG_1 count")? as usize;
    let expected = 4usize
        .checked_add(
            count
                .checked_mul(12)
                .ok_or_else(|| "DBG_1 大小溢出".to_string())?,
        )
        .ok_or_else(|| "DBG_1 大小溢出".to_string())?;
    if expected != data.len() {
        return Err(format!("DBG_1 大小不匹配: {} != {}", expected, data.len()));
    }
    let mut result = Vec::with_capacity(count);
    let mut previous = 0usize;
    for index in 0..count {
        let offset = read_u32(data, 4 + index * 12, "DBG_1 code offset")? as usize;
        let file_id = read_u32(data, 8 + index * 12, "DBG_1 file id")? as usize;
        let line = read_u32(data, 12 + index * 12, "DBG_1 line")?;
        if index > 0 && offset < previous {
            return Err(format!("DBG_1 code offset 回退于记录 {}", index));
        }
        if file_id >= file_count {
            return Err(format!("DBG_1 file id 越界于记录 {}", index));
        }
        previous = offset;
        result.push((offset, file_id, line));
    }
    Ok(result)
}

fn decode_instructions(
    code: &[u8],
) -> Result<(Vec<Instruction>, HashMap<usize, usize>), BundleError> {
    let mut result = Vec::new();
    let mut index = HashMap::new();
    let mut offset = 0usize;
    while offset < code.len() {
        let opcode = code[offset];
        let operand = if opcode >= 0x7e {
            if offset + 5 > code.len() {
                return Err(format!("CODE 宽指令于 0x{:x} 截断", offset));
            }
            Some(u32::from_le_bytes(
                code[offset + 1..offset + 5].try_into().unwrap(),
            ))
        } else {
            None
        };
        let current = result.len();
        index.insert(offset, current);
        result.push(Instruction {
            offset,
            opcode,
            operand,
        });
        offset += if operand.is_some() { 5 } else { 1 };
    }
    Ok((result, index))
}

fn validate_manifest(manifest: &Value) -> Result<usize, BundleError> {
    let format = manifest.get("format").and_then(Value::as_str);
    if !matches!(format, Some("AGSI_SB2_DUMP_SIMPLE_V1" | "AGSI_SB2_DUMP_V2")) {
        return Err(format!(
            "不支持的 SB2 解包目录格式: {}",
            format.unwrap_or("<missing>")
        ));
    }
    manifest
        .get("header_values")
        .and_then(Value::as_array)
        .and_then(|values| values.get(9))
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .ok_or_else(|| "manifest.header_values[9] 缺失或不是 CSTR 数量".to_string())
}

impl Bundle {
    pub fn load(root: &Path) -> Result<Self, BundleError> {
        let root = root
            .canonicalize()
            .map_err(|e| format!("输入目录 {} 不可用: {}", root.display(), e))?;
        if !root.is_dir() {
            return Err(format!("输入不是目录: {}", root.display()));
        }
        let manifest_text = fs::read_to_string(root.join("manifest.json"))
            .map_err(|e| format!("读取 manifest.json 失败: {}", e))?;
        let manifest: Value = serde_json::from_str(&manifest_text)
            .map_err(|e| format!("manifest.json 不是有效 JSON: {}", e))?;
        let cstr_count = validate_manifest(&manifest)?;
        let code = read_file(&root, "CODE.bin")?;
        let (instructions, instruction_index) = decode_instructions(&code)?;
        let cstr = parse_cstr(&read_file(&root, "CSTR.bin")?, cstr_count)?;
        let ftbl0 = parse_ftbl(&read_file(&root, "FTBL_0.bin")?, "FTBL_0")?;
        let ftbl1 = parse_ftbl(&read_file(&root, "FTBL_1.bin")?, "FTBL_1")?;
        let api_by_address = ftbl1
            .iter()
            .cloned()
            .map(|entry| (entry.address, entry))
            .collect();
        let source_files = parse_debug_files(&read_file(&root, "DBG_0.bin")?)?;
        let line_records = parse_debug_lines(&read_file(&root, "DBG_1.bin")?, source_files.len())?;

        let mut function_entries = BTreeMap::new();
        for insn in &instructions {
            if insn.opcode == 0xb5 {
                function_entries.insert(insn.offset, insn.operand.unwrap_or(0));
            }
        }
        if function_entries.is_empty() {
            return Err("CODE 没有 B5 函数入口".to_string());
        }
        let function_starts: Vec<usize> = function_entries.keys().copied().collect();
        let mut function_ranges = BTreeMap::new();
        for (index, start) in function_starts.iter().enumerate() {
            let end = function_starts
                .get(index + 1)
                .copied()
                .unwrap_or(code.len());
            let start_index = *instruction_index
                .get(start)
                .ok_or_else(|| format!("函数入口 0x{:x} 不是指令边界", start))?;
            let end_index = if end == code.len() {
                instructions.len()
            } else {
                *instruction_index
                    .get(&end)
                    .ok_or_else(|| format!("函数结束 0x{:x} 不是指令边界", end))?
            };
            let body = &instructions[start_index..end_index];
            if body.len() < 3
                || body[body.len() - 2].opcode != 0xb4
                || body[body.len() - 1].opcode != 0x79
            {
                return Err(format!("函数 0x{:x} 缺少 B4/79 结束", start));
            }
            function_ranges.insert(*start, (start_index, end_index));
        }
        Ok(Self {
            root,
            manifest,
            code,
            instructions,
            instruction_index,
            cstr,
            ftbl0,
            ftbl1,
            api_by_address,
            source_files,
            line_records,
            function_entries,
            function_ranges,
        })
    }

    pub fn source_info(&self, code_offset: usize) -> SourceInfo {
        let position = self
            .line_records
            .partition_point(|(offset, _, _)| *offset <= code_offset);
        let index = position.saturating_sub(1);
        let (_, file_id, line) = self.line_records[index];
        SourceInfo {
            file_id,
            file: self.source_files[file_id].clone(),
            line,
            dbg_record: index,
        }
    }

    pub fn function_start_for(&self, instruction_index: usize) -> usize {
        let offset = self.instructions[instruction_index].offset;
        self.function_ranges
            .range(..=offset)
            .next_back()
            .map(|(start, _)| *start)
            .unwrap_or(0)
    }

    pub fn function_instruction_range(&self, start: usize) -> (usize, usize) {
        self.function_ranges[&start]
    }

    pub fn argument_window(&self, call_index: usize, argc: usize) -> Option<&[Instruction]> {
        if call_index < argc {
            return None;
        }
        let args = &self.instructions[call_index - argc..call_index];
        let last = args.last()?;
        let width = if last.operand.is_some() { 5 } else { 1 };
        if last.offset + width != self.instructions[call_index].offset {
            return None;
        }
        Some(args)
    }

    pub fn cstr_text(&self, id: usize) -> Result<&str, BundleError> {
        self.cstr
            .get(id)
            .map(|entry| entry.text.as_str())
            .ok_or_else(|| format!("CSTR index {} 越界", id))
    }

    pub fn find_wrapper_by_apis(
        &self,
        required: &[&str],
        argc: usize,
    ) -> Result<usize, BundleError> {
        let targets: Vec<usize> = required
            .iter()
            .map(|name| {
                self.ftbl1
                    .iter()
                    .find(|entry| entry.name == *name)
                    .map(|entry| entry.address)
                    .ok_or_else(|| format!("FTBL_1 缺少 API {}", name))
            })
            .collect::<Result<_, _>>()?;
        let mut candidates = Vec::new();
        for (start, marker) in &self.function_entries {
            if *marker as usize != argc + 1 {
                continue;
            }
            let (begin, end) = self.function_ranges[start];
            let body = &self.instructions[begin..end];
            if targets.iter().all(|target| {
                body.iter()
                    .any(|insn| insn.opcode == 0xc6 && insn.operand == Some(*target as u32))
            }) {
                candidates.push(*start);
            }
        }
        if candidates.len() > 1 {
            let mut ranked = candidates
                .iter()
                .map(|target| {
                    let calls = self
                        .instructions
                        .iter()
                        .filter(|insn| insn.opcode == 0xb2 && insn.operand == Some(*target as u32))
                        .count();
                    (*target, calls)
                })
                .collect::<Vec<_>>();
            ranked.sort_by_key(|(_, calls)| std::cmp::Reverse(*calls));
            if ranked[0].1 > 0 && ranked.get(1).is_none_or(|second| ranked[0].1 > second.1) {
                return Ok(ranked[0].0);
            }
        }
        if candidates.len() != 1 {
            return Err(format!(
                "无法唯一识别文本包装函数: {} 个候选",
                candidates.len()
            ));
        }
        Ok(candidates[0])
    }

    pub fn resolve_frame_string(&self, load_index: usize) -> Option<(usize, usize)> {
        let load = self.instructions[load_index];
        if load.opcode != 0x87 {
            return None;
        }
        let function_start = self.function_start_for(load_index);
        let (begin, _) = self.function_ranges[&function_start];
        for index in (begin..load_index).rev() {
            let insn = self.instructions[index];
            if insn.opcode != 0x93 || insn.operand != load.operand {
                continue;
            }
            if index < begin + 3 {
                return None;
            }
            let assignment = &self.instructions[index - 3..=index];
            if assignment
                .iter()
                .map(|item| item.opcode)
                .collect::<Vec<_>>()
                != [0x82, 0x0c, 0x08, 0x93]
            {
                return None;
            }
            let cstr_id = assignment[0].operand? as usize;
            return Some((cstr_id, insn.offset));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_instructions, parse_cstr, swap_nibbles, validate_manifest};
    use serde_json::json;

    #[test]
    fn decodes_narrow_and_wide_instruction_boundaries() {
        let code = [0x01, 0x82, 0x34, 0x12, 0x00, 0x00, 0x79];
        let (instructions, index) = decode_instructions(&code).unwrap();
        assert_eq!(instructions.len(), 3);
        assert_eq!(instructions[0].offset, 0);
        assert_eq!(instructions[1].offset, 1);
        assert_eq!(instructions[1].operand, Some(0x1234));
        assert_eq!(instructions[2].offset, 6);
        assert_eq!(index[&6], 2);
    }

    #[test]
    fn rejects_truncated_wide_instruction() {
        assert!(decode_instructions(&[0x82, 1, 2, 3])
            .unwrap_err()
            .contains("截断"));
    }

    #[test]
    fn parses_contiguous_obfuscated_cstr_pool() {
        let first = b"A\0";
        let second = [0xf0, 0x49, 0];
        let mut data = Vec::new();
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&(first.len() as u32).to_le_bytes());
        data.extend_from_slice(&(first.len() as u32).to_le_bytes());
        data.extend_from_slice(&(second.len() as u32).to_le_bytes());
        let mut pool = first.to_vec();
        pool.extend_from_slice(&second);
        data.extend_from_slice(&swap_nibbles(&pool));

        let entries = parse_cstr(&data, 2).unwrap();
        assert_eq!(entries[0].text, "A");
        assert_eq!(entries[1].text, "\u{e009}");
        assert_eq!(entries[1].pool_offset, 2);
    }

    #[test]
    fn rejects_non_contiguous_cstr_offsets() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes());
        data.push(0);
        assert!(parse_cstr(&data, 1).unwrap_err().contains("不连续"));
    }

    #[test]
    fn accepts_both_supported_outer_manifest_formats() {
        for format in ["AGSI_SB2_DUMP_SIMPLE_V1", "AGSI_SB2_DUMP_V2"] {
            let manifest = json!({
                "format": format,
                "header_values": [0, 0, 0, 0, 0, 0, 0, 0, 0, 31910, 0]
            });
            assert_eq!(validate_manifest(&manifest).unwrap(), 31910);
        }
        assert!(validate_manifest(&json!({"format": "unknown"})).is_err());
    }
}
