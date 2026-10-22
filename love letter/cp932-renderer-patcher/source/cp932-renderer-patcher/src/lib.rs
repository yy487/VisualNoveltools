use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub const SOURCE_SIZE: usize = 262_144;
pub const SOURCE_SHA256: &str = "b9334fca270c138dc375e41ff708dab1c788237fe8cebf9ebd5161a2ae630f16";

const IMAGE_BASE: u32 = 0x0040_0000;
const GLYPH_IAT_VA: u32 = 0x0043_6040;
const MB_TO_WIDE_IAT_VA: u32 = 0x0043_60C4;
const HELPER_OFFSET: usize = 0x35_400;
const HELPER_VA: u32 = IMAGE_BASE + HELPER_OFFSET as u32;
const IMPORT_NAME_CHAR_OFFSET: usize = 0x37A67;
const LOAD_SITE_OFFSETS: [usize; 2] = [0x17_759, 0x17_7CB];
const CALL_SITE_OFFSET: usize = 0x17_948;
const HELPER_PADDING_SIZE: usize = 0xC00;

const EXPECTED_LOAD_BYTES: [u8; 6] = [0x8B, 0x35, 0x40, 0x60, 0x43, 0x00];
const EXPECTED_CALL_BYTES: [u8; 6] = [0xFF, 0x15, 0x40, 0x60, 0x43, 0x00];

#[derive(Debug, PartialEq, Eq)]
pub struct PatchReport {
    pub source_sha256: String,
    pub output_sha256: String,
    pub changed_bytes: usize,
    pub helper_bytes: usize,
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
    if source.len() < 0x40 || &source[..2] != b"MZ" {
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

fn append_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn patch_rel32(bytes: &mut [u8], displacement_offset: usize, target: usize) {
    let next = displacement_offset + 4;
    let displacement = (target as isize - next as isize) as i32;
    bytes[displacement_offset..displacement_offset + 4]
        .copy_from_slice(&displacement.to_le_bytes());
}

fn helper_code() -> Vec<u8> {
    let mut code = Vec::with_capacity(160);

    // The wrapper preserves the callee-saved registers and keeps the original
    // seven arguments at [ebp+08] through [ebp+20].
    code.extend_from_slice(&[0x55, 0x8B, 0xEC, 0x57, 0x56, 0x53, 0x83, 0xEC, 0x10]);
    code.extend_from_slice(&[0x8B, 0x45, 0x0C]);
    code.extend_from_slice(&[0x3D, 0xFF, 0x00, 0x00, 0x00]);
    let jbe_single = code.len();
    code.extend_from_slice(&[0x76, 0x00]);

    // A packed value above 0xFF is the original renderer's big-endian CP932
    // pair: high byte first, low byte second.
    code.extend_from_slice(&[0x8B, 0xD0, 0xC1, 0xE8, 0x08]);
    code.extend_from_slice(&[0x88, 0x45, 0xF0, 0x88, 0x55, 0xF1]);
    code.extend_from_slice(&[0xC7, 0x45, 0xF8, 0x02, 0x00, 0x00, 0x00]);
    let jmp_convert = code.len();
    code.extend_from_slice(&[0xEB, 0x00]);

    let single = code.len();
    code.extend_from_slice(&[0x88, 0x45, 0xF0]);
    code.extend_from_slice(&[0xC7, 0x45, 0xF8, 0x01, 0x00, 0x00, 0x00]);

    let convert = code.len();
    // MultiByteToWideChar(932, 0, source, length, &wide, 1).
    code.extend_from_slice(&[0x6A, 0x01]);
    code.extend_from_slice(&[0x8D, 0x4D, 0xF4, 0x51]);
    code.extend_from_slice(&[0xFF, 0x75, 0xF8]);
    code.extend_from_slice(&[0x8D, 0x4D, 0xF0, 0x51]);
    code.extend_from_slice(&[0x6A, 0x00, 0x68, 0xA4, 0x03, 0x00, 0x00]);
    code.extend_from_slice(&[0xFF, 0x15]);
    append_u32(&mut code, MB_TO_WIDE_IAT_VA);

    code.extend_from_slice(&[0x85, 0xC0]);
    let jle_fail = code.len();
    code.extend_from_slice(&[0x0F, 0x8E, 0x00, 0x00, 0x00, 0x00]);
    code.extend_from_slice(&[0x0F, 0xB7, 0x45, 0xF4]);

    // Call GetGlyphOutlineW with the original arguments and the converted
    // UTF-16 code unit in place of the packed CP932 value.
    code.extend_from_slice(&[0xFF, 0x75, 0x20]);
    code.extend_from_slice(&[0xFF, 0x75, 0x1C]);
    code.extend_from_slice(&[0xFF, 0x75, 0x18]);
    code.extend_from_slice(&[0xFF, 0x75, 0x14]);
    code.extend_from_slice(&[0xFF, 0x75, 0x10]);
    code.push(0x50);
    code.extend_from_slice(&[0xFF, 0x75, 0x08]);
    code.extend_from_slice(&[0xFF, 0x15]);
    append_u32(&mut code, GLYPH_IAT_VA);

    let epilogue = code.len();
    code.extend_from_slice(&[0x83, 0xC4, 0x10, 0x5B, 0x5E, 0x5F, 0x5D, 0xC2, 0x1C, 0x00]);

    let fail = code.len();
    code.extend_from_slice(&[0xB8, 0xFF, 0xFF, 0xFF, 0xFF]);
    let jmp_epilogue = code.len();
    code.extend_from_slice(&[0xE9, 0x00, 0x00, 0x00, 0x00]);

    let single_disp = (single as isize - (jbe_single + 2) as isize) as i8;
    code[jbe_single + 1] = single_disp as u8;
    let convert_disp = (convert as isize - (jmp_convert + 2) as isize) as i8;
    code[jmp_convert + 1] = convert_disp as u8;
    patch_rel32(&mut code, jle_fail + 2, fail);
    patch_rel32(&mut code, jmp_epilogue + 1, epilogue);
    code
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

pub fn prepare_patch(source: &[u8]) -> Result<(Vec<u8>, PatchReport), String> {
    if source.len() != SOURCE_SIZE {
        return Err(format!(
            "wrong input size: expected {SOURCE_SIZE} bytes, found {} bytes",
            source.len()
        ));
    }
    validate_pe32(source)?;

    let source_sha256 = sha256_hex(source);
    if source_sha256 != SOURCE_SHA256 {
        return Err(format!(
            "wrong input SHA-256: expected {SOURCE_SHA256}, found {source_sha256}"
        ));
    }
    if source[IMPORT_NAME_CHAR_OFFSET] != b'A' {
        return Err(format!(
            "GetGlyphOutline import name is not expected at 0x{IMPORT_NAME_CHAR_OFFSET:X}"
        ));
    }

    let helper = helper_code();
    if helper.len() > HELPER_PADDING_SIZE {
        return Err(format!(
            "helper is too large for the reserved code cave: {} bytes",
            helper.len()
        ));
    }
    let helper_end = HELPER_OFFSET + HELPER_PADDING_SIZE;
    if source[HELPER_OFFSET..helper_end]
        .iter()
        .any(|&byte| byte != 0)
    {
        return Err(format!(
            "reserved code cave at 0x{HELPER_OFFSET:X} is not zero padding"
        ));
    }

    let mut output = source.to_vec();
    output[IMPORT_NAME_CHAR_OFFSET] = b'W';

    for &offset in &LOAD_SITE_OFFSETS {
        let end = offset + EXPECTED_LOAD_BYTES.len();
        if source[offset..end] != EXPECTED_LOAD_BYTES {
            return Err(format!(
                "glyph helper load bytes do not match at 0x{offset:X}: expected [{}], found [{}]",
                hex_bytes(&EXPECTED_LOAD_BYTES),
                hex_bytes(&source[offset..end])
            ));
        }
        let mut replacement = [0x90; 6];
        replacement[0] = 0xBE;
        replacement[1..5].copy_from_slice(&HELPER_VA.to_le_bytes());
        output[offset..end].copy_from_slice(&replacement);
    }

    let call_end = CALL_SITE_OFFSET + EXPECTED_CALL_BYTES.len();
    if source[CALL_SITE_OFFSET..call_end] != EXPECTED_CALL_BYTES {
        return Err(format!(
            "glyph helper call bytes do not match at 0x{CALL_SITE_OFFSET:X}: expected [{}], found [{}]",
            hex_bytes(&EXPECTED_CALL_BYTES),
            hex_bytes(&source[CALL_SITE_OFFSET..call_end])
        ));
    }
    let relative = relative_call(CALL_SITE_OFFSET, HELPER_VA)?;
    output[CALL_SITE_OFFSET..CALL_SITE_OFFSET + 5].copy_from_slice(&relative);
    output[CALL_SITE_OFFSET + 5] = 0x90;

    output[HELPER_OFFSET..HELPER_OFFSET + helper.len()].copy_from_slice(&helper);

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
            helper_bytes: helper.len(),
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
        "import=GetGlyphOutlineW; cp932=932; helper=0x{HELPER_VA:X}; loads={:X},{:X}; call={:X}",
        LOAD_SITE_OFFSETS[0], LOAD_SITE_OFFSETS[1], CALL_SITE_OFFSET
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn matching_source() -> Vec<u8> {
        let source = fs::read(r"E:\GAL\love\loveletter_chs.exe")
            .expect("the supplied CHS sample is available");
        assert_eq!(source.len(), SOURCE_SIZE);
        assert_eq!(sha256_hex(&source), SOURCE_SHA256);
        source
    }

    #[test]
    fn helper_has_cp932_conversion_and_w_call() {
        let helper = helper_code();
        assert!(helper
            .windows(4)
            .any(|window| window == [0x68, 0xA4, 0x03, 0x00]));
        assert!(helper
            .windows(6)
            .any(|window| window == [0xFF, 0x15, 0xC4, 0x60, 0x43, 0x00]));
        assert!(helper
            .windows(6)
            .any(|window| window == [0xFF, 0x15, 0x40, 0x60, 0x43, 0x00]));
        assert!(!helper.ends_with(&[0xE9, 0, 0, 0, 0]));
        assert!(helper.len() < HELPER_PADDING_SIZE);
    }

    #[test]
    fn patches_only_the_renderer_sites_and_code_cave() {
        let source = matching_source();
        let (output, report) = prepare_patch(&source).expect("sample must patch");
        assert_eq!(output.len(), source.len());
        assert_eq!(report.helper_bytes, helper_code().len());
        assert_eq!(output[0xC747], 0xEB, "preserve the existing NoDVD branch");
        assert_eq!(
            &output[IMPORT_NAME_CHAR_OFFSET - 15..IMPORT_NAME_CHAR_OFFSET + 1],
            b"GetGlyphOutlineW"
        );
        for &offset in &LOAD_SITE_OFFSETS {
            assert_eq!(
                &output[offset..offset + 2],
                &[0xBE, (HELPER_VA & 0xFF) as u8]
            );
            assert_eq!(output[offset + 5], 0x90);
        }
        assert_eq!(output[CALL_SITE_OFFSET], 0xE8);
        assert_eq!(output[CALL_SITE_OFFSET + 5], 0x90);
        assert_eq!(
            &output[HELPER_OFFSET..HELPER_OFFSET + helper_code().len()],
            helper_code().as_slice()
        );
        assert!(source
            .iter()
            .zip(&output)
            .enumerate()
            .all(|(offset, (before, after))| {
                let in_helper =
                    (HELPER_OFFSET..HELPER_OFFSET + helper_code().len()).contains(&offset);
                let in_import = offset == IMPORT_NAME_CHAR_OFFSET;
                let in_load = LOAD_SITE_OFFSETS
                    .iter()
                    .any(|&site| (site..site + EXPECTED_LOAD_BYTES.len()).contains(&offset));
                let in_call = (CALL_SITE_OFFSET..CALL_SITE_OFFSET + EXPECTED_CALL_BYTES.len())
                    .contains(&offset);
                in_helper || in_import || in_load || in_call || before == after
            }));
    }

    #[test]
    fn rejects_wrong_hash() {
        let mut source = matching_source();
        source[0x200] ^= 1;
        let error = prepare_patch(&source).expect_err("hash mismatch must fail");
        assert!(error.contains("wrong input SHA-256"));
    }
}
