//! Deterministic CP932 carrier allocation shared by script and font stages.

use std::collections::{BTreeMap, BTreeSet};

const RANGES: [(u32, u32, u32); 6] = [
    (0, 0x20, 0x7e),
    (1, 0xa1, 0xdf),
    (2, 0x8140, 0x84be),
    (3, 0x889f, 0x9872),
    (4, 0x989f, 0x9ffc),
    (5, 0xe040, 0xeaa4),
];

pub fn slot_for_code(code: u16, mask: u32) -> Option<usize> {
    let mut slot = 0usize;
    for (bit, start, end) in RANGES {
        if mask & (1 << bit) == 0 {
            continue;
        }
        if u32::from(code) >= start && u32::from(code) <= end {
            return Some(slot + usize::from(code) - start as usize);
        }
        slot += (end - start + 1) as usize;
    }
    None
}

pub fn code_for_slot(target: usize, mask: u32) -> Option<u16> {
    let mut slot = 0usize;
    for (bit, start, end) in RANGES {
        if mask & (1 << bit) == 0 {
            continue;
        }
        let len = (end - start + 1) as usize;
        if target < slot + len {
            return Some((start as usize + target - slot) as u16);
        }
        slot += len;
    }
    None
}

#[cfg(windows)]
mod cp932 {
    use std::ptr;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn WideCharToMultiByte(
            cp: u32,
            flags: u32,
            wide: *const u16,
            wide_len: i32,
            out: *mut u8,
            out_len: i32,
            default: *const u8,
            used_default: *mut i32,
        ) -> i32;
        fn MultiByteToWideChar(
            cp: u32,
            flags: u32,
            bytes: *const u8,
            byte_len: i32,
            out: *mut u16,
            out_len: i32,
        ) -> i32;
    }
    pub fn encode(ch: char) -> Option<u16> {
        if ch as u32 > 0xffff {
            return None;
        }
        let mut out = [0u8; 4];
        let mut used = 0;
        let source = [ch as u32 as u16];
        let n = unsafe {
            WideCharToMultiByte(
                932,
                0x400,
                source.as_ptr(),
                1,
                out.as_mut_ptr(),
                4,
                ptr::null(),
                &mut used,
            )
        };
        if used != 0 || !(1..=2).contains(&n) {
            return None;
        }
        Some(if n == 1 {
            u16::from(out[0])
        } else {
            u16::from_be_bytes([out[0], out[1]])
        })
    }
    pub fn decode(code: u16) -> Option<char> {
        let bytes = if code <= 0xff {
            [code as u8, 0]
        } else {
            code.to_be_bytes()
        };
        let nbytes = if code <= 0xff { 1 } else { 2 };
        let mut out = [0u16; 2];
        let n = unsafe { MultiByteToWideChar(932, 8, bytes.as_ptr(), nbytes, out.as_mut_ptr(), 2) };
        if n != 1 {
            return None;
        }
        char::from_u32(u32::from(out[0]))
    }
}

#[cfg(windows)]
pub fn cp932_encode(ch: char) -> Option<u16> {
    cp932::encode(ch)
}
#[cfg(windows)]
pub fn cp932_decode(code: u16) -> Option<char> {
    cp932::decode(code)
}

#[cfg(not(windows))]
pub fn cp932_encode(_ch: char) -> Option<u16> {
    None
}
#[cfg(not(windows))]
pub fn cp932_decode(_code: u16) -> Option<char> {
    None
}

pub fn decode_text(bytes: &[u8]) -> Result<String, String> {
    let mut text = String::new();
    let mut at = 0;
    while at < bytes.len() {
        let first = bytes[at];
        let lead = (0x81..=0x9f).contains(&first) || (0xe0..=0xfc).contains(&first);
        let (code, used) = if lead {
            let second = *bytes.get(at + 1).ok_or("CP932 双字节字符被截断")?;
            (u16::from_be_bytes([first, second]), 2)
        } else {
            (u16::from(first), 1)
        };
        let ch = cp932_decode(code).ok_or_else(|| format!("无效 CP932 码 {code:04X}"))?;
        text.push(ch);
        at += used;
    }
    Ok(text)
}

pub fn codes_in_text(bytes: &[u8]) -> Result<Vec<u16>, String> {
    let mut codes = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let first = bytes[at];
        let lead = (0x81..=0x9f).contains(&first) || (0xe0..=0xfc).contains(&first);
        let (code, used) = if lead {
            let second = *bytes.get(at + 1).ok_or("CP932 双字节字符被截断")?;
            (u16::from_be_bytes([first, second]), 2)
        } else {
            (u16::from(first), 1)
        };
        if cp932_decode(code).is_none() {
            return Err(format!("无效 CP932 码 {code:04X}"));
        }
        codes.push(code);
        at += used;
    }
    Ok(codes)
}

/// Reserves every representable character in the final source corpus, then
/// maps unrepresentable/out-of-range translation characters one-to-one.
pub fn plan_carriers(
    mask: u32,
    preserved_text: impl IntoIterator<Item = impl AsRef<str>>,
    translated_text: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<BTreeMap<char, u16>, String> {
    plan_carriers_with_codes(mask, preserved_text, std::iter::empty(), translated_text)
}

pub fn plan_carriers_with_codes(
    mask: u32,
    preserved_text: impl IntoIterator<Item = impl AsRef<str>>,
    raw_preserved_codes: impl IntoIterator<Item = u16>,
    translated_text: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<BTreeMap<char, u16>, String> {
    let mut occupied = BTreeSet::new();
    for text in preserved_text {
        for ch in text.as_ref().chars() {
            if let Some(code) = cp932_encode(ch) {
                if let Some(slot) = slot_for_code(code, mask) {
                    occupied.insert(slot);
                }
            }
        }
    }
    for code in raw_preserved_codes {
        if let Some(slot) = slot_for_code(code, mask) {
            occupied.insert(slot);
        }
    }
    let mut required = BTreeSet::new();
    for text in translated_text {
        for ch in text.as_ref().chars() {
            if let Some(code) = cp932_encode(ch) {
                if cp932_decode(code) == Some(ch) {
                    if let Some(slot) = slot_for_code(code, mask) {
                        occupied.insert(slot);
                        continue;
                    }
                }
            }
            {
                required.insert(ch);
            }
        }
    }
    let mut result = BTreeMap::new();
    let mut free = Vec::new();
    let mut index = 0usize;
    for (bit, start, end) in RANGES {
        if mask & (1 << bit) == 0 {
            continue;
        }
        for code in start..=end {
            let code = code as u16;
            if bit != 0 && cp932_decode(code).is_some() && !occupied.contains(&index) {
                free.push((index, code));
            }
            index += 1;
        }
    }
    if required.len() > free.len() {
        return Err(format!(
            "载体容量不足：需 {} 个有效槽，剩余 {}",
            required.len(),
            free.len()
        ));
    }
    for (ch, (_, code)) in required.into_iter().zip(free) {
        result.insert(ch, code);
    }
    Ok(result)
}

/// Encodes UTF-8 text to native CP932 bytes, applying the shared carrier plan.
pub fn encode_text(text: &str, carriers: &BTreeMap<char, u16>) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    for ch in text.chars() {
        let code = carriers
            .get(&ch)
            .copied()
            .or_else(|| cp932_encode(ch))
            .ok_or_else(|| format!("字符 U+{:04X} 没有 CP932 载体", ch as u32))?;
        if code <= 0xff {
            out.push(code as u8);
        } else {
            out.extend_from_slice(&code.to_be_bytes());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn range_indices_follow_the_six_mask_segments() {
        assert_eq!(slot_for_code(0x20, 0x3d), Some(0));
        assert_eq!(slot_for_code(0x7e, 0x3d), Some(94));
        assert_eq!(slot_for_code(0x8140, 0x3d), Some(95));
        assert_eq!(slot_for_code(0x7f, 0x3d), None);
        assert_eq!(
            slot_for_code(0xe040, 0x3d),
            Some(9589 - (0xeaa4 - 0xe040 + 1))
        );
    }

    #[cfg(windows)]
    #[test]
    fn native_translation_slots_are_reserved_before_carriers() {
        let native = cp932_encode('　').unwrap();
        assert_eq!(native, 0x8140);
        let map = plan_carriers(0x3d, [""], ["你　"]).unwrap();
        assert_ne!(map[&'你'], native);
    }
}
