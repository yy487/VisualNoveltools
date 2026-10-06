use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 8] = b"RepiPack";
const HEADER_STATE: u32 = (-2_088_779_654i32) as u32;
const HEADER_MIX: u32 = (-1_728_259_166i32) as u32;
const ENTRY_STATE: u32 = (-182_998_490i32) as u32;
const ENTRY_MIX: u32 = 0x98fc_dba2;
// The v2 index stores an 80-byte record: a 64-byte name, then DWORD offset,
// unpacked size and packed size, followed by a one-byte data transform tag.
const ENTRY_SIZE: usize = 0x50;
const NAME_SIZE: usize = 0x40;

pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub raw_name: Vec<u8>,
    pub name: String,
    pub offset: u32,
    pub packed_size: u32,
    pub unpacked_size: u32,
    pub crypt: u8,
    pub reserved: [u8; 3],
}

#[derive(Clone, Debug)]
pub struct Archive {
    pub header_name_raw: Vec<u8>,
    pub header_name: String,
    pub entries: Vec<Entry>,
    table_end: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RepackStats {
    pub entries: usize,
    pub changed: usize,
    pub compressed_changed: usize,
    pub raw_changed: usize,
    pub preserved_packed: usize,
}

struct StoredEntry {
    raw_name: Vec<u8>,
    packed: Vec<u8>,
    unpacked_size: u32,
    crypt: u8,
    reserved: [u8; 3],
}

impl Archive {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 {
            return Err("文件短于 RepiPack v2 最小头部".into());
        }
        if &bytes[..8] != MAGIC {
            return Err("不是 RepiPack 文件：头部签名不匹配".into());
        }
        let version = read_u32(bytes, 8)?;
        if version != 2 {
            return Err(format!("只支持 RepiPack v2，文件版本为 {version}"));
        }

        let header_name_size = read_u32(bytes, 12)? as usize;
        if header_name_size == 0 || header_name_size > 4096 {
            return Err(format!("头部文件名长度无效: {header_name_size}"));
        }
        let header_name_end = 16usize
            .checked_add(header_name_size)
            .ok_or("头部文件名长度溢出")?;
        if header_name_end > bytes.len() {
            return Err("头部文件名超出文件末尾".into());
        }
        let mut header_name_raw =
            decode_rolling(&bytes[16..header_name_end], HEADER_STATE, HEADER_MIX);
        if let Some(nul) = header_name_raw.iter().position(|byte| *byte == 0) {
            header_name_raw.truncate(nul);
        }
        if header_name_raw.is_empty() {
            return Err("头部文件名解码后为空".into());
        }
        let header_name = decode_cp932(&header_name_raw);

        let count = read_u32(bytes, header_name_end)? as usize;
        let table_start = header_name_end.checked_add(4).ok_or("表起始位置溢出")?;
        let table_size = count.checked_mul(ENTRY_SIZE).ok_or("表大小溢出")?;
        let table_end = table_start
            .checked_add(table_size)
            .ok_or("表结束位置溢出")?;
        if table_end > bytes.len() {
            return Err(format!(
                "表越界：需要到 0x{table_end:X}，文件只有 0x{:X}",
                bytes.len()
            ));
        }

        let mut entries = Vec::new();
        entries
            .try_reserve(count)
            .map_err(|_| "RepiPack 表项数量无法分配")?;
        for index in 0..count {
            let start = table_start + index * ENTRY_SIZE;
            let decoded = decode_rolling(&bytes[start..start + ENTRY_SIZE], ENTRY_STATE, ENTRY_MIX);
            let name_field = &decoded[..NAME_SIZE];
            let name_end = name_field
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(name_field.len());
            if name_end == 0 {
                return Err(format!("表项 {index} 的资源名为空"));
            }
            let raw_name = name_field[..name_end].to_vec();
            let offset = read_u32(&decoded, NAME_SIZE)?;
            let unpacked_size = read_u32(&decoded, NAME_SIZE + 4)?;
            let packed_size = read_u32(&decoded, NAME_SIZE + 8)?;
            let crypt = decoded[NAME_SIZE + 12];
            let data_start = offset as usize;
            let data_end = data_start
                .checked_add(packed_size as usize)
                .ok_or_else(|| format!("表项 {index} 数据范围溢出"))?;
            if data_start < table_end || data_end > bytes.len() {
                return Err(format!(
                    "表项 {index} 数据越界: offset=0x{offset:X}, packed={packed_size}"
                ));
            }
            entries.push(Entry {
                name: decode_cp932(&raw_name),
                raw_name,
                offset,
                packed_size,
                unpacked_size,
                crypt,
                reserved: decoded[77..80].try_into().unwrap(),
            });
        }

        Ok(Self {
            header_name_raw,
            header_name,
            entries,
            table_end,
        })
    }

    pub fn unpack_entry(&self, bytes: &[u8], entry: &Entry) -> Result<Vec<u8>> {
        if self.table_end > bytes.len() {
            return Err("输入在预检后发生变化".into());
        }
        let start = entry.offset as usize;
        let end = start
            .checked_add(entry.packed_size as usize)
            .ok_or("packed 数据范围溢出")?;
        let source = bytes.get(start..end).ok_or("packed 数据超出输入文件")?;
        let mut packed = source.to_vec();
        match entry.crypt {
            1 => packed = decode_rolling(&packed, ENTRY_STATE, HEADER_MIX),
            2 => {
                for byte in &mut packed {
                    *byte = byte.rotate_left(6) ^ 0x26;
                }
            }
            _ => {}
        }
        if entry.packed_size == entry.unpacked_size {
            return Ok(packed);
        }
        lzss_decompress(&packed, entry.unpacked_size as usize)
    }

    pub fn packed_slice<'a>(&self, bytes: &'a [u8], entry: &Entry) -> Result<&'a [u8]> {
        let start = entry.offset as usize;
        let end = start
            .checked_add(entry.packed_size as usize)
            .ok_or("packed 数据范围溢出")?;
        bytes
            .get(start..end)
            .ok_or_else(|| "packed 数据超出输入文件".into())
    }

    /// Rebuild an archive from a full extracted directory, preserving original
    /// packed bytes for unchanged entries and storing edited entries raw.
    pub fn repack_from_directory(
        &self,
        original: &[u8],
        input_dir: &Path,
    ) -> Result<(Vec<u8>, RepackStats)> {
        let input_root = fs::canonicalize(input_dir)
            .map_err(|error| format!("无法读取回封来源目录 {}: {error}", input_dir.display()))?;
        if !input_root.is_dir() {
            return Err(format!("回封来源不是目录: {}", input_root.display()));
        }
        let mut source_files = collect_source_files(&input_root)?;
        let mut stored = Vec::with_capacity(self.entries.len());
        let mut seen_names = HashSet::new();
        let mut stats = RepackStats {
            entries: self.entries.len(),
            ..RepackStats::default()
        };

        for (index, entry) in self.entries.iter().enumerate() {
            let components = safe_components(&entry.name)
                .map_err(|error| format!("归档表项 {index} 路径无效: {error}"))?;
            let relative: PathBuf = components.iter().collect();
            let key = path_key(&relative);
            if !seen_names.insert(key.clone()) {
                return Err(format!("归档中存在重复资源路径: {}", entry.name));
            }
            let source_path = source_files
                .remove(&key)
                .ok_or_else(|| format!("回封来源缺少归档资源: {}", entry.name))?;
            let replacement = fs::read(&source_path)
                .map_err(|error| format!("读取回封文件 {} 失败: {error}", source_path.display()))?;
            let original_plain = self.unpack_entry(original, entry)?;
            if replacement == original_plain {
                stored.push(StoredEntry {
                    raw_name: entry.raw_name.clone(),
                    packed: self.packed_slice(original, entry)?.to_vec(),
                    unpacked_size: entry.unpacked_size,
                    crypt: entry.crypt,
                    reserved: entry.reserved,
                });
                stats.preserved_packed += 1;
            } else {
                if replacement.len() > u32::MAX as usize {
                    return Err(format!("回封资源过大: {}", entry.name));
                }
                let compressed = lzss_compress(&replacement);
                let (packed, crypt) = if compressed.len() < replacement.len() {
                    let check = lzss_decompress(&compressed, replacement.len())?;
                    if check != replacement {
                        return Err(format!("新压缩数据往返不一致: {}", entry.name));
                    }
                    stats.compressed_changed += 1;
                    (compressed, 0)
                } else {
                    stats.raw_changed += 1;
                    (replacement.clone(), 0)
                };
                stored.push(StoredEntry {
                    raw_name: entry.raw_name.clone(),
                    packed,
                    unpacked_size: replacement.len() as u32,
                    crypt,
                    reserved: entry.reserved,
                });
                stats.changed += 1;
            }
        }

        if !source_files.is_empty() {
            let unexpected = source_files.values().next().unwrap();
            return Err(format!(
                "回封来源含有不属于该归档的文件: {}",
                unexpected.display()
            ));
        }

        let trailing_start = self
            .entries
            .iter()
            .map(|entry| entry.offset as usize + entry.packed_size as usize)
            .max()
            .unwrap_or(self.table_end);
        let trailing = original.get(trailing_start..).unwrap_or_default();
        let packed = build_archive_v2(&self.header_name_raw, &stored, trailing)?;
        Ok((packed, stats))
    }
}

fn collect_source_files(root: &Path) -> Result<HashMap<String, PathBuf>> {
    fn visit(root: &Path, directory: &Path, files: &mut HashMap<String, PathBuf>) -> Result<()> {
        for entry in fs::read_dir(directory)
            .map_err(|error| format!("读取目录 {} 失败: {error}", directory.display()))?
        {
            let path = entry
                .map_err(|error| format!("枚举回封来源失败: {error}"))?
                .path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("读取文件状态 {} 失败: {error}", path.display()))?;
            if metadata.file_type().is_symlink() {
                return Err(format!("回封来源不接受符号链接: {}", path.display()));
            }
            if metadata.is_dir() {
                visit(root, &path, files)?;
            } else if metadata.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|_| format!("回封文件不在来源目录内: {}", path.display()))?;
                let key = path_key(relative);
                if files.insert(key.clone(), path).is_some() {
                    return Err(format!("回封来源存在不区分大小写的重名: {key}"));
                }
            } else {
                return Err(format!("回封来源含有非普通文件: {}", path.display()));
            }
        }
        Ok(())
    }

    let mut files = HashMap::new();
    visit(root, root, &mut files)?;
    Ok(files)
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

fn build_archive_v2(
    header_name: &[u8],
    entries: &[StoredEntry],
    trailing: &[u8],
) -> Result<Vec<u8>> {
    if header_name.is_empty() || header_name.len() > 4096 {
        return Err("包头文件名为空或过长".into());
    }
    if entries.len() > u32::MAX as usize {
        return Err("RepiPack 条目数超过 DWORD 范围".into());
    }
    let header_end = 16usize
        .checked_add(header_name.len())
        .ok_or("头部名称长度溢出")?;
    let table_start = header_end.checked_add(4).ok_or("表起始位置溢出")?;
    let table_size = entries.len().checked_mul(ENTRY_SIZE).ok_or("表大小溢出")?;
    let table_end = table_start
        .checked_add(table_size)
        .ok_or("表结束位置溢出")?;
    if table_end > u32::MAX as usize {
        return Err("RepiPack 索引超过 DWORD 偏移范围".into());
    }

    let mut offset = table_end;
    let mut records = Vec::with_capacity(table_size);
    for entry in entries {
        if entry.raw_name.is_empty()
            || entry.raw_name.len() >= NAME_SIZE
            || entry.raw_name.contains(&0)
        {
            return Err("资源路径长度无效或包含 NUL".into());
        }
        if entry.packed.len() > u32::MAX as usize {
            return Err("资源大小超过 DWORD 范围".into());
        }
        let end = offset
            .checked_add(entry.packed.len())
            .ok_or("归档总长度溢出")?;
        if end > u32::MAX as usize {
            return Err("归档超过 4 GiB，无法写入 DWORD 偏移".into());
        }
        let mut record = [0u8; ENTRY_SIZE];
        record[..entry.raw_name.len()].copy_from_slice(&entry.raw_name);
        record[NAME_SIZE..NAME_SIZE + 4].copy_from_slice(&(offset as u32).to_le_bytes());
        record[NAME_SIZE + 4..NAME_SIZE + 8].copy_from_slice(&entry.unpacked_size.to_le_bytes());
        record[NAME_SIZE + 8..NAME_SIZE + 12]
            .copy_from_slice(&(entry.packed.len() as u32).to_le_bytes());
        record[NAME_SIZE + 12] = entry.crypt;
        record[NAME_SIZE + 13..NAME_SIZE + 16].copy_from_slice(&entry.reserved);
        records.extend_from_slice(&encode_rolling(&record, ENTRY_STATE, ENTRY_MIX));
        offset = end;
    }

    let final_size = offset
        .checked_add(trailing.len())
        .ok_or("尾部数据长度溢出")?;
    let mut output = Vec::new();
    output
        .try_reserve(final_size)
        .map_err(|_| "RepiPack 输出长度无法分配")?;
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&2u32.to_le_bytes());
    output.extend_from_slice(&(header_name.len() as u32).to_le_bytes());
    output.extend_from_slice(&encode_rolling(header_name, HEADER_STATE, HEADER_MIX));
    output.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    output.extend_from_slice(&records);
    for entry in entries {
        output.extend_from_slice(&entry.packed);
    }
    output.extend_from_slice(trailing);
    Ok(output)
}

fn encode_rolling(bytes: &[u8], initial: u32, mix: u32) -> Vec<u8> {
    let mut output = bytes.to_vec();
    let mut state = initial;
    for offset in (0..bytes.len() / 4).map(|index| index * 4) {
        let plain = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let encrypted = plain ^ state;
        output[offset..offset + 4].copy_from_slice(&encrypted.to_le_bytes());
        state = state.wrapping_add(plain.rotate_left(16) ^ mix);
    }
    output
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let end = offset.checked_add(4).ok_or("DWORD 位置溢出")?;
    let raw = bytes
        .get(offset..end)
        .ok_or_else(|| format!("文件在 0x{offset:X} 处截断"))?;
    Ok(u32::from_le_bytes(raw.try_into().unwrap()))
}

fn decode_rolling(bytes: &[u8], initial: u32, mix: u32) -> Vec<u8> {
    let mut output = bytes.to_vec();
    let mut state = initial;
    for offset in (0..bytes.len() / 4).map(|index| index * 4) {
        let encrypted = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let plain = encrypted ^ state;
        output[offset..offset + 4].copy_from_slice(&plain.to_le_bytes());
        state = state.wrapping_add(plain.rotate_left(16) ^ mix);
    }
    output
}

fn lzss_decompress(input: &[u8], expected: usize) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    output
        .try_reserve(expected)
        .map_err(|_| "LZSS 输出长度无法分配")?;
    let mut window = [0u8; 0x1000];
    let mut write_pos = 4078usize;
    let mut cursor = 0usize;
    let mut flags = 0u16;
    while output.len() < expected {
        flags >>= 1;
        if flags & 0x100 == 0 {
            let Some(&flag) = input.get(cursor) else {
                return Ok(output);
            };
            cursor += 1;
            flags = u16::from(flag) | 0xff00;
        }
        if flags & 1 != 0 {
            let Some(&byte) = input.get(cursor) else {
                return Ok(output);
            };
            cursor += 1;
            output.push(byte);
            window[write_pos] = byte;
            write_pos = (write_pos + 1) & 0xfff;
        } else {
            let Some(&first) = input.get(cursor) else {
                return Ok(output);
            };
            let Some(&second) = input.get(cursor + 1) else {
                return Ok(output);
            };
            cursor += 2;
            let source = (((second as usize) & 0xf0) << 4) | first as usize;
            let length = (second as usize & 0x0f) + 3;
            for index in 0..length {
                if output.len() >= expected {
                    break;
                }
                let byte = window[(source + index) & 0xfff];
                output.push(byte);
                window[write_pos] = byte;
                write_pos = (write_pos + 1) & 0xfff;
            }
        }
    }
    Ok(output)
}

fn lzss_compress(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut chains: HashMap<[u8; 3], VecDeque<usize>> = HashMap::new();
    let mut next_insert = 0usize;
    let mut cursor = 0usize;

    while cursor < input.len() {
        let flags_offset = output.len();
        output.push(0);
        let mut flags = 0u8;

        for bit in 0..8 {
            if cursor >= input.len() {
                break;
            }
            let remaining = input.len() - cursor;
            let max_length = remaining.min(18);
            let mut best_length = 0usize;
            let mut best_source = 0usize;

            if remaining >= 3 {
                while next_insert + 2 < cursor {
                    let key = [
                        input[next_insert],
                        input[next_insert + 1],
                        input[next_insert + 2],
                    ];
                    chains.entry(key).or_default().push_back(next_insert);
                    next_insert += 1;
                }

                // The game's LZSS ring starts as zeroes at write position 4078.
                if cursor < 0x1000 {
                    let virtual_source = cursor as isize - 0x1000;
                    let length = lzss_match(input, cursor, virtual_source, max_length);
                    if length > best_length {
                        best_length = length;
                        best_source = (4078isize + virtual_source).rem_euclid(0x1000) as usize;
                    }
                }

                // Very short distances are not in the three-byte hash chain yet.
                for distance in 1..=cursor.min(2) {
                    let source_position = cursor - distance;
                    let length = lzss_match(input, cursor, source_position as isize, max_length);
                    if length > best_length {
                        best_length = length;
                        best_source = (4078 + source_position) & 0xfff;
                    }
                }

                let key = [input[cursor], input[cursor + 1], input[cursor + 2]];
                if let Some(candidates) = chains.get_mut(&key) {
                    while candidates
                        .front()
                        .is_some_and(|position| cursor - *position > 0x1000)
                    {
                        candidates.pop_front();
                    }
                    for source_position in candidates.iter().rev().take(96) {
                        let length =
                            lzss_match(input, cursor, *source_position as isize, max_length);
                        if length > best_length {
                            best_length = length;
                            best_source = (4078 + *source_position) & 0xfff;
                            if length == max_length {
                                break;
                            }
                        }
                    }
                }
            }

            if best_length >= 3 {
                let first = best_source as u8;
                let second = (((best_source >> 4) & 0xf0) as u8) | (best_length as u8 - 3);
                output.push(first);
                output.push(second);
                cursor += best_length;
            } else {
                flags |= 1 << bit;
                output.push(input[cursor]);
                cursor += 1;
            }
        }
        output[flags_offset] = flags;
    }
    output
}

fn lzss_match(input: &[u8], cursor: usize, source: isize, max_length: usize) -> usize {
    let mut length = 0usize;
    while length < max_length {
        let source_position = source + length as isize;
        let source_byte = if source_position < 0 {
            0
        } else if let Some(byte) = input.get(source_position as usize) {
            *byte
        } else {
            break;
        };
        if source_byte != input[cursor + length] {
            break;
        }
        length += 1;
    }
    length
}

pub fn decode_cp932(bytes: &[u8]) -> String {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn MultiByteToWideChar(
                code_page: u32,
                flags: u32,
                source: *const i8,
                source_len: i32,
                destination: *mut u16,
                destination_len: i32,
            ) -> i32;
        }
        let required = unsafe {
            MultiByteToWideChar(
                932,
                0,
                bytes.as_ptr() as *const i8,
                bytes.len() as i32,
                std::ptr::null_mut(),
                0,
            )
        };
        if required > 0 {
            let mut wide = vec![0u16; required as usize];
            let written = unsafe {
                MultiByteToWideChar(
                    932,
                    0,
                    bytes.as_ptr() as *const i8,
                    bytes.len() as i32,
                    wide.as_mut_ptr(),
                    required,
                )
            };
            if written == required {
                return OsString::from_wide(&wide).to_string_lossy().into_owned();
            }
        }
    }
    String::from_utf8_lossy(bytes).into_owned()
}

pub fn encode_cp932(text: &str) -> Result<Vec<u8>> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn WideCharToMultiByte(
                code_page: u32,
                flags: u32,
                source: *const u16,
                source_len: i32,
                destination: *mut i8,
                destination_len: i32,
                default_char: *const i8,
                used_default: *mut i32,
            ) -> i32;
        }
        let wide: Vec<u16> = std::ffi::OsStr::new(text).encode_wide().collect();
        if wide.is_empty() {
            return Ok(Vec::new());
        }
        let source_len = i32::try_from(wide.len()).map_err(|_| "CP932 输入过长")?;
        let default_char = b"?";
        let mut used_default = 0;
        let required = unsafe {
            WideCharToMultiByte(
                932,
                0x0000_0400,
                wide.as_ptr(),
                source_len,
                std::ptr::null_mut(),
                0,
                default_char.as_ptr() as *const i8,
                &mut used_default,
            )
        };
        if required <= 0 || used_default != 0 {
            return Err("译文包含无法编码为 CP932 的字符".into());
        }
        let mut bytes = vec![0u8; required as usize];
        used_default = 0;
        let written = unsafe {
            WideCharToMultiByte(
                932,
                0x0000_0400,
                wide.as_ptr(),
                source_len,
                bytes.as_mut_ptr() as *mut i8,
                required,
                default_char.as_ptr() as *const i8,
                &mut used_default,
            )
        };
        if written != required || used_default != 0 {
            return Err("译文包含无法编码为 CP932 的字符".into());
        }
        Ok(bytes)
    }
    #[cfg(not(windows))]
    {
        if text.is_ascii() {
            Ok(text.as_bytes().to_vec())
        } else {
            Err("CP932 转码目前仅在 Windows 主机上可用".into())
        }
    }
}

pub fn safe_components(name: &str) -> Result<Vec<String>> {
    if name.is_empty() || name.chars().any(|ch| ch.is_control()) {
        return Err("资源路径为空或含控制字符".into());
    }
    if name.starts_with('/') || name.starts_with('\\') || name.contains(':') {
        return Err("资源路径是绝对路径或包含盘符".into());
    }
    let mut components = Vec::new();
    for component in name.replace('\\', "/").split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            return Err("资源路径包含父目录跳转".into());
        }
        if component
            .chars()
            .any(|ch| matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
            || component.ends_with('.')
            || component.ends_with(' ')
        {
            return Err("资源路径包含 Windows 不安全文件名字符".into());
        }
        let device_stem = component
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        let reserved = matches!(
            device_stem.as_str(),
            "CON"
                | "PRN"
                | "AUX"
                | "NUL"
                | "COM1"
                | "COM2"
                | "COM3"
                | "COM4"
                | "COM5"
                | "COM6"
                | "COM7"
                | "COM8"
                | "COM9"
                | "LPT1"
                | "LPT2"
                | "LPT3"
                | "LPT4"
                | "LPT5"
                | "LPT6"
                | "LPT7"
                | "LPT8"
                | "LPT9"
        );
        if reserved {
            return Err("资源路径包含 Windows 保留设备名".into());
        }
        components.push(component.to_owned());
    }
    if components.is_empty() {
        return Err("资源路径不是安全的相对路径".into());
    }
    Ok(components)
}

pub fn source_file_name_matches(archive: &Archive, path: &Path) -> bool {
    path.file_name()
        .map(|name| name.to_string_lossy() == archive.header_name)
        .unwrap_or(false)
}
