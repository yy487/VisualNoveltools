use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub mod faeries_script;

const MAGIC: &[u8; 8] = b"RepiPack";
const HEADER_STATE: u32 = 0x6c84_d7fb;
const RECORD_STATE: u32 = 0xf17b_254a;
const MIX: u32 = 0x5b1e_3078;
const DATA_KEY_LIMIT: usize = 0x400;

pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub hash: [u8; 16],
    pub offset: u32,
    pub packed_size: u32,
    pub unpacked_size: u32,
    pub flags: u32,
}

#[derive(Clone, Debug)]
pub struct Archive {
    pub header_name: String,
    pub version: u32,
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug)]
pub struct PackItem {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct NameCandidate {
    pub raw: Vec<u8>,
    pub display: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepiProfile {
    pub version: u8,
    pub key_index: u32,
    pub mix: u32,
    pub header_state: u32,
    pub record_state: u32,
}

const VERSION5_PROFILES: [(u32, u32, u32); 17] = [
    (0x35a3_21b3, 0xaf50_1792, 0xd75c_9255),
    (0x37c1_ade2, 0x7d59_cab4, 0x805f_2cad),
    (0xdc5d_affd, 0x5d08_be67, 0x33af_5162),
    (0x48d5_fa21, 0x3729_fbb4, 0x8c82_0f5d),
    (0x8b4e_16d9, 0x361f_8aa5, 0x5c92_0f8a),
    (0xce49_85a0, 0x8af4_622e, 0x1af5_6cdb),
    (0xaa58_e9bd, 0x7e5c_b068, 0x502a_2cf5),
    (0x38a4_e237, 0x7620_89bd, 0xf5ab_5bdc),
    (0xfa87_1ab2, 0xded5_a4d1, 0x8128_cfa5),
    (0x77df_8518, 0xbd42_d5ef, 0xf358_ace6),
    (0xa54f_18bf, 0xed57_0357, 0x4a2b_81dc),
    (0x6b7a_c18d, 0x1a4d_8052, 0x4057_8245),
    (0x5b1e_3078, 0x6c84_d7fb, 0xf17b_254a),
    (0x501a_bbcf, 0x49d2_1831, 0xc6af_b354),
    (0xcc54_5b24, 0x8305_afbe, 0xae71_06fb),
    (0xef15_7364, 0x7c0d_b4a3, 0x5a8f_bcc2),
    (0x57cc_4ec0, 0xdb36_37a8, 0xfb4a_25f1),
];

pub fn repipack_profile(version: &str, key_index: u32) -> Result<RepiProfile> {
    if version != "5" {
        return Err(
            "当前已确认可回封的 arc_conv 配置只有 version=5；其他版本使用不同数据密码流程".into(),
        );
    }
    let (mix, header_state, record_state) = *VERSION5_PROFILES
        .get(key_index as usize)
        .ok_or_else(|| format!("version=5 的 key_index 超出范围: {key_index}（可用 0..16）"))?;
    Ok(RepiProfile {
        version: 5,
        key_index,
        mix,
        header_state,
        record_state,
    })
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let end = offset.checked_add(4).ok_or("整数位置溢出")?;
    let part = bytes
        .get(offset..end)
        .ok_or_else(|| format!("文件在 0x{offset:X} 处截断"))?;
    Ok(u32::from_le_bytes([part[0], part[1], part[2], part[3]]))
}

fn decode_rolling(bytes: &[u8], initial: u32) -> Vec<u8> {
    let mut output = bytes.to_vec();
    let mut state = initial;
    for index in 0..bytes.len() / 4 {
        let offset = index * 4;
        let encrypted = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let plain = encrypted ^ state;
        output[offset..offset + 4].copy_from_slice(&plain.to_le_bytes());
        state = state.wrapping_add(plain.rotate_left(16) ^ MIX);
    }
    output
}

fn encode_rolling_with_mix(bytes: &[u8], initial: u32, mix: u32) -> Vec<u8> {
    let mut output = bytes.to_vec();
    let mut state = initial;
    for index in 0..bytes.len() / 4 {
        let offset = index * 4;
        let plain = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let encrypted = plain ^ state;
        output[offset..offset + 4].copy_from_slice(&encrypted.to_le_bytes());
        state = state.wrapping_add(plain.rotate_left(16) ^ mix);
    }
    output
}

impl Archive {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 {
            return Err("文件短于 RepiPack 最小头部".into());
        }
        if &bytes[..8] != MAGIC {
            return Err("不是 RepiPack 文件：头部签名不匹配".into());
        }
        let version = read_u32(bytes, 8)?;
        if version != 5 {
            return Err(format!("不支持的 RepiPack 版本: {version}"));
        }

        let header_size = read_u32(bytes, 12)? as usize;
        let header_end = 16usize.checked_add(header_size).ok_or("头部长度溢出")?;
        if header_size > 1 << 20 || header_end > bytes.len() {
            return Err(format!("头部长度越界: {header_size} 字节"));
        }
        let decoded_header = decode_rolling(&bytes[16..header_end], HEADER_STATE);
        let header_name = String::from_utf8_lossy(&decoded_header)
            .trim_end_matches('\0')
            .to_owned();

        let count = read_u32(bytes, header_end)? as usize;
        let table_start = header_end.checked_add(4).ok_or("表起始位置溢出")?;
        let table_size = count.checked_mul(32).ok_or("表大小溢出")?;
        let table_end = table_start
            .checked_add(table_size)
            .ok_or("表结束位置溢出")?;
        if table_end > bytes.len() {
            return Err(format!(
                "表越界：需要到 0x{table_end:X}，文件只有 0x{:X}",
                bytes.len()
            ));
        }

        let mut entries = Vec::with_capacity(count);
        for index in 0..count {
            let start = table_start + index * 32;
            let decoded = decode_rolling(&bytes[start..start + 32], RECORD_STATE);
            let mut hash = [0u8; 16];
            hash.copy_from_slice(&decoded[..16]);
            let offset = u32::from_le_bytes(decoded[16..20].try_into().unwrap());
            let packed_size = u32::from_le_bytes(decoded[20..24].try_into().unwrap());
            let unpacked_size = u32::from_le_bytes(decoded[24..28].try_into().unwrap());
            let flags = u32::from_le_bytes(decoded[28..32].try_into().unwrap());
            let data_start = offset as usize;
            let data_end = data_start
                .checked_add(packed_size as usize)
                .ok_or_else(|| format!("表项 {index} 的数据范围溢出"))?;
            if data_start < table_end || data_end > bytes.len() {
                return Err(format!(
                    "表项 {index} 数据越界: offset=0x{offset:X}, packed={packed_size}"
                ));
            }
            entries.push(Entry {
                hash,
                offset,
                packed_size,
                unpacked_size,
                flags,
            });
        }
        Ok(Self {
            header_name,
            version,
            entries,
        })
    }
}

/// 游戏先取 basename，再按 CP932 的单字节规则小写化。
pub fn normalized_name(name: &str) -> Vec<u8> {
    normalized_name_bytes(&encode_cp932(name))
}

pub fn normalized_name_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut start = 0;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if is_sjis_lead(byte) {
            index = (index + 2).min(bytes.len());
        } else {
            index += 1;
            if byte == b'/' || byte == b'\\' {
                start = index;
            }
        }
    }

    let mut normalized = Vec::with_capacity(bytes.len() - start);
    index = start;
    while index < bytes.len() {
        let byte = bytes[index];
        if is_sjis_lead(byte) {
            normalized.push(byte);
            if index + 1 < bytes.len() {
                normalized.push(bytes[index + 1]);
            }
            index += 2;
        } else {
            normalized.push(byte.to_ascii_lowercase());
            index += 1;
        }
    }
    normalized
}

pub fn md5_name(name: &str) -> [u8; 16] {
    md5_name_bytes(&encode_cp932(name))
}

pub fn md5_name_bytes(name: &[u8]) -> [u8; 16] {
    md5(&normalized_name_bytes(name))
}

fn is_sjis_lead(byte: u8) -> bool {
    (0x81..=0x9f).contains(&byte) || (0xe0..=0xfc).contains(&byte)
}

pub fn encode_cp932(text: &str) -> Vec<u8> {
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
        let required = unsafe {
            WideCharToMultiByte(
                932,
                0,
                wide.as_ptr(),
                wide.len() as i32,
                std::ptr::null_mut(),
                0,
                std::ptr::null(),
                std::ptr::null_mut(),
            )
        };
        if required > 0 {
            let mut bytes = vec![0u8; required as usize];
            let written = unsafe {
                WideCharToMultiByte(
                    932,
                    0,
                    wide.as_ptr(),
                    wide.len() as i32,
                    bytes.as_mut_ptr() as *mut i8,
                    required,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                )
            };
            if written == required {
                return bytes;
            }
        }
    }
    text.as_bytes().to_vec()
}

const MD5_S: [[u32; 4]; 4] = [
    [7, 12, 17, 22],
    [5, 9, 14, 20],
    [4, 11, 16, 23],
    [6, 10, 15, 21],
];
const MD5_K: [u32; 64] = [
    0xd76a_a478,
    0xe8c7_b756,
    0x2420_70db,
    0xc1bd_ceee,
    0xf57c_0faf,
    0x4787_c62a,
    0xa830_4613,
    0xfd46_9501,
    0x6980_98d8,
    0x8b44_f7af,
    0xffff_5bb1,
    0x895c_d7be,
    0x6b90_1122,
    0xfd98_7193,
    0xa679_438e,
    0x49b4_0821,
    0xf61e_2562,
    0xc040_b340,
    0x265e_5a51,
    0xe9b6_c7aa,
    0xd62f_105d,
    0x0244_1453,
    0xd8a1_e681,
    0xe7d3_fbc8,
    0x21e1_cde6,
    0xc337_07d6,
    0xf4d5_0d87,
    0x455a_14ed,
    0xa9e3_e905,
    0xfcef_a3f8,
    0x676f_02d9,
    0x8d2a_4c8a,
    0xfffa_3942,
    0x8771_f681,
    0x6d9d_6122,
    0xfde5_380c,
    0xa4be_ea44,
    0x4bde_cfa9,
    0xf6bb_4b60,
    0xbebf_bc70,
    0x289b_7ec6,
    0xeaa1_27fa,
    0xd4ef_3085,
    0x0488_1d05,
    0xd9d4_d039,
    0xe6db_99e5,
    0x1fa2_7cf8,
    0xc4ac_5665,
    0xf429_2244,
    0x432a_ff97,
    0xab94_23a7,
    0xfc93_a039,
    0x655b_59c3,
    0x8f0c_cc92,
    0xffef_f47d,
    0x8584_5dd1,
    0x6fa8_7e4f,
    0xfe2c_e6e0,
    0xa301_4314,
    0x4e08_11a1,
    0xf753_7e82,
    0xbd3a_f235,
    0x2ad7_d2bb,
    0xeb86_d391,
];

fn md5_transform(state: &mut [u32; 4], chunk: &[u8]) {
    debug_assert_eq!(chunk.len(), 64);
    let mut words = [0u32; 16];
    for (index, word) in words.iter_mut().enumerate() {
        let offset = index * 4;
        *word = u32::from_le_bytes(chunk[offset..offset + 4].try_into().unwrap());
    }
    let (mut a, mut b, mut c, mut d) = (state[0], state[1], state[2], state[3]);
    for index in 0..64 {
        let (function, word_index) = match index {
            0..=15 => ((b & c) | ((!b) & d), index),
            16..=31 => ((d & b) | ((!d) & c), (5 * index + 1) % 16),
            32..=47 => (b ^ c ^ d, (3 * index + 5) % 16),
            _ => (c ^ (b | !d), (7 * index) % 16),
        };
        let rotated = a
            .wrapping_add(function)
            .wrapping_add(MD5_K[index])
            .wrapping_add(words[word_index])
            .rotate_left(MD5_S[index / 16][index % 4]);
        a = d;
        d = c;
        c = b;
        b = b.wrapping_add(rotated);
    }
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
}

fn md5_digest(state: &[u32; 4]) -> [u8; 16] {
    let mut digest = [0u8; 16];
    for (index, word) in state.iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    digest
}

fn md5(input: &[u8]) -> [u8; 16] {
    let mut length = input.len() + 9;
    length += (64 - length % 64) % 64;
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut padded = vec![0u8; length];
    padded[..input.len()].copy_from_slice(input);
    padded[input.len()] = 0x80;
    padded[length - 8..].copy_from_slice(&bit_len.to_le_bytes());

    let mut state = [0x6745_2301u32, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476];
    for chunk in padded.chunks_exact(64) {
        md5_transform(&mut state, chunk);
    }
    md5_digest(&state)
}

/// RepiPack updates one MD5 state repeatedly, padding each suffix call in place.
struct RepiMd5 {
    state: [u32; 4],
    bit_count: u64,
}

impl RepiMd5 {
    fn new() -> Self {
        Self {
            state: [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476],
            bit_count: 0,
        }
    }

    fn update_and_finalize(&mut self, input: &[u8]) -> [u8; 16] {
        self.bit_count = self
            .bit_count
            .wrapping_add((input.len() as u64).wrapping_mul(8));

        let full_len = input.len() & !63;
        for chunk in input[..full_len].chunks_exact(64) {
            md5_transform(&mut self.state, chunk);
        }

        let remainder = &input[full_len..];
        let tail_len = if remainder.len() < 56 { 64 } else { 128 };
        let mut tail = vec![0u8; tail_len];
        tail[..remainder.len()].copy_from_slice(remainder);
        tail[remainder.len()] = 0x80;
        tail[tail_len - 8..].copy_from_slice(&self.bit_count.to_le_bytes());
        for chunk in tail.chunks_exact(64) {
            md5_transform(&mut self.state, chunk);
        }
        md5_digest(&self.state)
    }
}

fn data_key(name: &str) -> Result<Vec<u8>> {
    data_key_bytes(name.as_bytes())
}

fn data_key_bytes(name: &[u8]) -> Result<Vec<u8>> {
    let normalized = normalized_name_bytes(name);
    if normalized.is_empty() {
        return Err("条目名称为空，无法生成数据密钥".into());
    }
    let reversed: Vec<u8> = normalized.iter().rev().copied().collect();
    let mut key = Vec::with_capacity(DATA_KEY_LIMIT);
    let mut context = RepiMd5::new();
    for index in 0..64usize {
        let digest = context.update_and_finalize(&reversed[index % reversed.len()..]);
        key.extend_from_slice(&digest);
    }
    Ok(key)
}

pub fn extract_entry(bytes: &[u8], entry: &Entry, name: &str) -> Result<Vec<u8>> {
    extract_entry_bytes(bytes, entry, name.as_bytes())
}

pub fn extract_entry_bytes(bytes: &[u8], entry: &Entry, name: &[u8]) -> Result<Vec<u8>> {
    let offset = entry.offset as usize;
    let packed_size = entry.packed_size as usize;
    let unpacked_size = entry.unpacked_size as usize;
    let end = offset.checked_add(packed_size).ok_or("数据范围溢出")?;
    if end > bytes.len() {
        return Err("数据范围超出文件末尾".into());
    }
    let mut packed = bytes[offset..end].to_vec();
    let key = data_key_bytes(name)?;
    for (byte, key_byte) in packed.iter_mut().take(DATA_KEY_LIMIT).zip(key.iter()) {
        *byte ^= *key_byte;
    }
    if packed_size == unpacked_size {
        return Ok(packed);
    }
    lzss_decompress(&packed, unpacked_size)
}

/// Build a valid RepiPack v5 archive using raw (uncompressed) entries.
/// Raw entries are accepted by the game's reader when packed_size == unpacked_size.
pub fn pack_archive(header_name: &str, items: &[PackItem]) -> Result<Vec<u8>> {
    let profile = repipack_profile("5", 12)?;
    pack_archive_with_profile(header_name, items, profile)
}

pub fn pack_archive_with_profile(
    header_name: &str,
    items: &[PackItem],
    profile: RepiProfile,
) -> Result<Vec<u8>> {
    if items.len() > u32::MAX as usize {
        return Err("条目数量超过 DWORD 范围".into());
    }
    let header = header_name.as_bytes();
    if header.is_empty() || header.len() > 1 << 20 {
        return Err("封包头名称为空或过长".into());
    }

    let table_start = 16usize
        .checked_add(header.len())
        .and_then(|value| value.checked_add(4))
        .ok_or("表起始位置溢出")?;
    let table_size = items.len().checked_mul(32).ok_or("表大小溢出")?;
    let data_start = table_start
        .checked_add(table_size)
        .ok_or("数据起始位置溢出")?;
    let mut offset = data_start;
    let mut entries = Vec::with_capacity(items.len());
    let mut encrypted_data = Vec::with_capacity(items.len());
    let mut hashes = std::collections::HashSet::new();

    for item in items {
        let hash = md5_name(&item.name);
        if !hashes.insert(hash) {
            return Err(format!("逻辑文件名重复或 basename 冲突: {}", item.name));
        }
        let size = item.data.len();
        let end = offset.checked_add(size).ok_or("封包数据大小溢出")?;
        if end > u32::MAX as usize {
            return Err("封包超过 4 GiB，无法写入 DWORD 偏移".into());
        }
        let mut data = item.data.clone();
        let key = data_key(&item.name)?;
        for (byte, key_byte) in data.iter_mut().take(DATA_KEY_LIMIT).zip(key.iter()) {
            *byte ^= *key_byte;
        }
        entries.push(Entry {
            hash,
            offset: offset as u32,
            packed_size: size as u32,
            unpacked_size: size as u32,
            flags: 0,
        });
        encrypted_data.push(data);
        offset = end;
    }

    let mut output = Vec::with_capacity(offset);
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&(profile.version as u32).to_le_bytes());
    output.extend_from_slice(&(header.len() as u32).to_le_bytes());
    output.extend_from_slice(&encode_rolling_with_mix(
        header,
        profile.header_state,
        profile.mix,
    ));
    output.extend_from_slice(&(items.len() as u32).to_le_bytes());

    for entry in &entries {
        let mut record = Vec::with_capacity(32);
        record.extend_from_slice(&entry.hash);
        record.extend_from_slice(&entry.offset.to_le_bytes());
        record.extend_from_slice(&entry.packed_size.to_le_bytes());
        record.extend_from_slice(&entry.unpacked_size.to_le_bytes());
        record.extend_from_slice(&entry.flags.to_le_bytes());
        output.extend_from_slice(&encode_rolling_with_mix(
            &record,
            profile.record_state,
            profile.mix,
        ));
    }
    for data in encrypted_data {
        output.extend_from_slice(&data);
    }
    Ok(output)
}

fn lzss_decompress(input: &[u8], expected: usize) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(expected);
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

pub fn hash_hex(hash: &[u8; 16]) -> String {
    let mut text = String::with_capacity(32);
    for byte in hash {
        use std::fmt::Write;
        write!(&mut text, "{byte:02x}").unwrap();
    }
    text
}

/// Stable content fingerprint used to bind translation files to their source scripts.
pub fn content_fingerprint(bytes: &[u8]) -> String {
    hash_hex(&md5(bytes))
}

pub fn load_name_candidates(path: &Path) -> Result<HashMap<[u8; 16], NameCandidate>> {
    let bytes =
        fs::read(path).map_err(|error| format!("读取名称表失败 {}: {error}", path.display()))?;
    parse_name_candidates(&bytes)
}

pub fn built_in_name_candidates() -> Result<HashMap<[u8; 16], NameCandidate>> {
    let mut result = HashMap::new();
    for (line_number, raw_line) in include_str!("faeries_script_names.txt").lines().enumerate() {
        let display = raw_line.trim();
        if display.is_empty() || display.starts_with('#') {
            continue;
        }
        let raw = encode_cp932(display);
        if raw.is_empty() {
            return Err(format!("内置文件名第 {} 行编码后为空", line_number + 1));
        }
        result.entry(md5_name_bytes(&raw)).or_insert(NameCandidate {
            raw,
            display: display.to_owned(),
        });
    }
    Ok(result)
}

fn parse_name_candidates(bytes: &[u8]) -> Result<HashMap<[u8; 16], NameCandidate>> {
    let mut result = HashMap::new();
    for (line_number, raw_line) in bytes.split(|byte| *byte == b'\n').enumerate() {
        let mut line = raw_line;
        if line.last() == Some(&b'\r') {
            line = &line[..line.len() - 1];
        }
        while line.first().is_some_and(|byte| byte.is_ascii_whitespace()) {
            line = &line[1..];
        }
        while line.last().is_some_and(|byte| byte.is_ascii_whitespace()) {
            line = &line[..line.len() - 1];
        }
        if line.is_empty() || line.first() == Some(&b'#') {
            continue;
        }
        let name = if let Some(tab) = line.iter().position(|byte| *byte == b'\t') {
            let (hash, candidate) = line.split_at(tab);
            if hash.len() == 32 && hash.iter().all(|byte| byte.is_ascii_hexdigit()) {
                &candidate[1..]
            } else {
                line
            }
        } else {
            line
        };
        if name.is_empty() {
            return Err(format!("名称表第 {} 行为空名称", line_number + 1));
        }
        result
            .entry(md5_name_bytes(name))
            .or_insert_with(|| NameCandidate {
                raw: name.to_vec(),
                display: decode_cp932(name),
            });
    }
    Ok(result)
}

fn decode_cp932(bytes: &[u8]) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[test]
    fn md5_matches_reference_vectors() {
        assert_eq!(hex(&md5(b"")), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(
            hex(&md5(b"adventure.txt")),
            "cf17acb41dc551b1330d8a1899caf92d"
        );
        assert_eq!(
            hex(&md5(b"The quick brown fox jumps over the lazy dog")),
            "9e107d9d372bb6826bd81d3542a419d6"
        );
    }

    #[test]
    fn names_use_lowercase_basename() {
        assert_eq!(
            normalized_name(r"FSGeneral\Script\Adventure.TXT"),
            b"adventure.txt"
        );
        assert_eq!(
            md5_name("FSGeneral/Script/Adventure.TXT"),
            md5_name("adventure.txt")
        );
    }

    #[test]
    fn pack_and_extract_round_trip() {
        let original = b"round trip payload\0with bytes".to_vec();
        let packed = pack_archive(
            "Script.dat",
            &[PackItem {
                name: "Adventure.TXT".into(),
                data: original.clone(),
            }],
        )
        .expect("pack");
        let archive = Archive::parse(&packed).expect("parse");
        assert_eq!(archive.entries.len(), 1);
        assert_eq!(
            extract_entry(&packed, &archive.entries[0], "adventure.txt").expect("extract"),
            original
        );
    }
}
