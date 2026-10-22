use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub const SOURCE_SIZE: usize = 262_144;
pub const SOURCE_SHA256: &str = "f40b55c12116496cd94c317a02fe8b8f72db0e81aadb35a8226a31ddfae25926";
pub const NODVD_SOURCE_SHA256: &str =
    "b9334fca270c138dc375e41ff708dab1c788237fe8cebf9ebd5161a2ae630f16";

const IMAGE_BASE: u32 = 0x0040_0000;
const FUNCTION_VA: u32 = 0x0041_4D20;
const CAVE_OFFSET: usize = 0x1D_811;
const CAVE_SIZE: usize = 15;
const CAVE_ORIGINAL: [u8; CAVE_SIZE] = [0x90; CAVE_SIZE];
const CALL_OFFSETS: [usize; 6] = [0x6_27E, 0x15_9AC, 0x20_CC8, 0x20_E89, 0x21_108, 0x21_2B9];
const EXPECTED_CALL_BYTES: [[u8; 5]; 6] = [
    [0xE8, 0x9D, 0xEA, 0x00, 0x00],
    [0xE8, 0x6F, 0xF3, 0xFF, 0xFF],
    [0xE8, 0x53, 0x40, 0xFF, 0xFF],
    [0xE8, 0x92, 0x3E, 0xFF, 0xFF],
    [0xE8, 0x13, 0x3C, 0xFF, 0xFF],
    [0xE8, 0x62, 0x3A, 0xFF, 0xFF],
];

#[derive(Debug, PartialEq, Eq)]
pub struct PatchReport {
    pub source_sha256: String,
    pub output_sha256: String,
    pub changed_bytes: usize,
    pub patched_calls: usize,
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn validate_pe32(source: &[u8]) -> Result<(), String> {
    if source.len() < 0x100 || &source[..2] != b"MZ" {
        return Err("input is not an MZ executable".to_owned());
    }
    let pe_offset = u32::from_le_bytes(
        source[0x3C..0x40]
            .try_into()
            .map_err(|_| "input has a truncated DOS header")?,
    ) as usize;
    let signature_end = pe_offset
        .checked_add(26)
        .ok_or_else(|| "PE header offset overflow".to_owned())?;
    if signature_end > source.len() || &source[pe_offset..pe_offset + 4] != b"PE\0\0" {
        return Err("input has no valid PE signature".to_owned());
    }
    if u16::from_le_bytes([source[pe_offset + 4], source[pe_offset + 5]]) != 0x014C {
        return Err("input is not an i386 PE".to_owned());
    }
    if u16::from_le_bytes([source[pe_offset + 24], source[pe_offset + 25]]) != 0x010B {
        return Err("input is not a PE32 executable".to_owned());
    }
    Ok(())
}

fn relative_call(site_offset: usize, target_va: u32) -> Result<[u8; 5], String> {
    let next_va = u64::from(IMAGE_BASE)
        .checked_add((site_offset + 5) as u64)
        .ok_or_else(|| "call-site address overflow".to_owned())?;
    let displacement = i64::from(target_va) - next_va as i64;
    let displacement = i32::try_from(displacement)
        .map_err(|_| format!("relative call displacement out of range at 0x{site_offset:X}"))?;
    let mut bytes = [0u8; 5];
    bytes[0] = 0xE8;
    bytes[1..].copy_from_slice(&displacement.to_le_bytes());
    Ok(bytes)
}

fn guard_code() -> Result<[u8; CAVE_SIZE], String> {
    let mut code = [0u8; CAVE_SIZE];
    // If the source image slot contains 0 or 1, clean the seven stack
    // arguments and return. Otherwise jump directly into the original callee.
    code[..5].copy_from_slice(&[0x83, 0x7C, 0x24, 0x0C, 0x01]);
    code[5..7].copy_from_slice(&[0x76, 0x05]);
    code[7] = 0xE9;
    let next_va = u64::from(IMAGE_BASE) + (CAVE_OFFSET as u64) + 12;
    let displacement = i64::from(FUNCTION_VA) - next_va as i64;
    let displacement = i32::try_from(displacement)
        .map_err(|_| "guard jump displacement out of range".to_owned())?;
    code[8..12].copy_from_slice(&displacement.to_le_bytes());
    code[12..].copy_from_slice(&[0xC2, 0x1C, 0x00]);
    Ok(code)
}

pub fn prepare_patch(source: &[u8]) -> Result<(Vec<u8>, PatchReport), String> {
    if source.len() != SOURCE_SIZE {
        return Err(format!(
            "wrong input size: expected {SOURCE_SIZE} bytes, found {} bytes",
            source.len()
        ));
    }
    validate_pe32(source)?;

    let source_sha256 = sha256_hex(source);
    if source_sha256 != SOURCE_SHA256 && source_sha256 != NODVD_SOURCE_SHA256 {
        return Err(format!(
            "wrong input SHA-256: expected {SOURCE_SHA256} or {NODVD_SOURCE_SHA256}, found {source_sha256}"
        ));
    }

    let cave_end = CAVE_OFFSET + CAVE_SIZE;
    if source[cave_end - CAVE_SIZE..cave_end] != CAVE_ORIGINAL {
        return Err(format!(
            "guard cave at 0x{CAVE_OFFSET:X} is not the expected NOP padding"
        ));
    }

    let guard = guard_code()?;
    let mut output = source.to_vec();
    output[CAVE_OFFSET..cave_end].copy_from_slice(&guard);

    for (index, &offset) in CALL_OFFSETS.iter().enumerate() {
        let end = offset + EXPECTED_CALL_BYTES[index].len();
        let actual = &source[offset..end];
        if actual != EXPECTED_CALL_BYTES[index] {
            return Err(format!(
                "call-site bytes do not match at 0x{offset:X}: expected [{}], found [{}]",
                hex_bytes(&EXPECTED_CALL_BYTES[index]),
                hex_bytes(actual)
            ));
        }
        let replacement = relative_call(offset, IMAGE_BASE + CAVE_OFFSET as u32)?;
        output[offset..end].copy_from_slice(&replacement);
    }

    let changed_bytes = source
        .iter()
        .zip(&output)
        .filter(|(before, after)| before != after)
        .count();
    let output_sha256 = sha256_hex(&output);
    Ok((
        output,
        PatchReport {
            source_sha256,
            output_sha256,
            changed_bytes,
            patched_calls: CALL_OFFSETS.len(),
        },
    ))
}

pub fn write_new_verified(output_path: &Path, output: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path)
        .map_err(|error| format!("cannot create {}: {error}", output_path.display()))?;
    if let Err(error) = file.write_all(output).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(output_path);
        return Err(format!("cannot write {}: {error}", output_path.display()));
    }
    drop(file);
    let written = fs::read(output_path).map_err(|error| {
        let _ = fs::remove_file(output_path);
        format!("cannot verify output {}: {error}", output_path.display())
    })?;
    if written != output {
        let _ = fs::remove_file(output_path);
        return Err(format!(
            "output verification failed; incomplete file was removed: {}",
            output_path.display()
        ));
    }
    Ok(())
}

pub fn patch_description() -> String {
    format!(
        "guard=0x{CAVE_OFFSET:X}; calls={}; source-slot<=1 is skipped",
        CALL_OFFSETS
            .iter()
            .map(|offset| format!("0x{offset:X}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn matching_source() -> Vec<u8> {
        let source =
            fs::read(r"E:\GAL\love\loveletter.exe").expect("the supplied real sample is available");
        assert_eq!(source.len(), SOURCE_SIZE);
        assert_eq!(sha256_hex(&source), SOURCE_SHA256);
        source
    }

    fn matching_nodvd_source() -> Vec<u8> {
        let source = fs::read(r"E:\GAL\love\loveletter_chs.exe")
            .expect("the supplied NoDVD-modified sample is available");
        assert_eq!(source.len(), SOURCE_SIZE);
        assert_eq!(sha256_hex(&source), NODVD_SOURCE_SHA256);
        assert_eq!(source[0xC747], 0xEB);
        source
    }

    #[test]
    fn patches_only_guard_cave_and_known_calls() {
        let source = matching_source();
        let (output, report) = prepare_patch(&source).expect("sample must patch");
        assert_eq!(output.len(), source.len());
        assert_eq!(report.patched_calls, 6);
        assert_eq!(report.changed_bytes, 30);
        assert_eq!(report.output_sha256, sha256_hex(&output));
        assert_eq!(
            &output[CAVE_OFFSET..CAVE_OFFSET + 7],
            &[0x83, 0x7C, 0x24, 0x0C, 0x01, 0x76, 0x05]
        );
        assert!(source
            .iter()
            .zip(&output)
            .enumerate()
            .all(|(offset, (before, after))| {
                let in_cave = (CAVE_OFFSET..CAVE_OFFSET + CAVE_SIZE).contains(&offset);
                let in_call = CALL_OFFSETS
                    .iter()
                    .any(|&site| (site..site + 5).contains(&offset));
                in_cave || in_call || before == after
            }));
    }

    #[test]
    fn rejects_wrong_hash() {
        let mut source = matching_source();
        source[0x200] ^= 1;
        let error = prepare_patch(&source).expect_err("hash mismatch must fail");
        assert!(error.contains("wrong input SHA-256"));
    }

    #[test]
    fn accepts_nodvd_source_and_preserves_nodvd_branch() {
        let source = matching_nodvd_source();
        let (output, report) = prepare_patch(&source).expect("NoDVD sample must patch");
        assert_eq!(report.source_sha256, NODVD_SOURCE_SHA256);
        assert_eq!(output[0xC747], 0xEB);
        assert_eq!(report.changed_bytes, 30);
        assert_eq!(output.len(), source.len());
    }

    #[test]
    fn guard_code_has_expected_stack_contract() {
        let code = guard_code().expect("guard displacement must fit");
        assert_eq!(&code[..7], &[0x83, 0x7C, 0x24, 0x0C, 0x01, 0x76, 0x05]);
        assert_eq!(&code[12..], &[0xC2, 0x1C, 0x00]);
    }
}
