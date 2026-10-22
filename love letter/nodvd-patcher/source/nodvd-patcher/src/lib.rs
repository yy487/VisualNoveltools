use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub const SOURCE_SIZE: usize = 262_144;
pub const SOURCE_SHA256: &str = "f40b55c12116496cd94c317a02fe8b8f72db0e81aadb35a8226a31ddfae25926";

const PATCH_OFFSET: usize = 0xC745;
const EXPECTED_BYTES: &[u8] = &[0x85, 0xC0, 0x75, 0x1D, 0xA1, 0x30, 0xF6, 0x43, 0x00];
const REPLACEMENT_BYTES: &[u8] = &[0x85, 0xC0, 0xEB, 0x1D, 0xA1, 0x30, 0xF6, 0x43, 0x00];

#[derive(Debug, PartialEq, Eq)]
pub struct PatchReport {
    pub source_sha256: String,
    pub output_sha256: String,
    pub changed_bytes: usize,
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

    let end = PATCH_OFFSET + EXPECTED_BYTES.len();
    let actual = &source[PATCH_OFFSET..end];
    if actual != EXPECTED_BYTES {
        return Err(format!(
            "original bytes do not match at 0x{PATCH_OFFSET:X}: expected [{}], found [{}]",
            hex_bytes(EXPECTED_BYTES),
            hex_bytes(actual)
        ));
    }

    let mut output = source.to_vec();
    output[PATCH_OFFSET..end].copy_from_slice(REPLACEMENT_BYTES);
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

    let written = match fs::read(output_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            let _ = fs::remove_file(output_path);
            return Err(format!(
                "cannot verify output {}: {error}",
                output_path.display()
            ));
        }
    };
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
        "offset=0x{PATCH_OFFSET:X} original=[{}] replacement=[{}]",
        hex_bytes(EXPECTED_BYTES),
        hex_bytes(REPLACEMENT_BYTES)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matching_source() -> Vec<u8> {
        let source =
            fs::read(r"E:\GAL\love\loveletter.exe").expect("the supplied real sample is available");
        assert_eq!(source.len(), SOURCE_SIZE);
        assert_eq!(sha256_hex(&source), SOURCE_SHA256);
        source
    }

    #[test]
    fn patches_only_the_disc_gate_branch() {
        let source = matching_source();
        let (output, report) = prepare_patch(&source).expect("sample must patch");

        assert_eq!(output.len(), source.len());
        assert_eq!(report.changed_bytes, 1);
        assert_eq!(report.source_sha256, SOURCE_SHA256);
        assert_eq!(
            report.output_sha256,
            "b9334fca270c138dc375e41ff708dab1c788237fe8cebf9ebd5161a2ae630f16"
        );
        assert_eq!(&output[PATCH_OFFSET..PATCH_OFFSET + 9], REPLACEMENT_BYTES);
        assert!(source
            .iter()
            .zip(&output)
            .enumerate()
            .all(|(offset, (before, after))| offset == 0xC747 || before == after));
    }

    #[test]
    fn rejects_wrong_hash() {
        let mut source = matching_source();
        source[0x200] ^= 1;
        let error = prepare_patch(&source).expect_err("hash mismatch must fail");
        assert!(error.contains("wrong input SHA-256"));
    }

    #[test]
    fn rejects_wrong_size() {
        let mut source = matching_source();
        source.pop();
        let error = prepare_patch(&source).expect_err("size mismatch must fail");
        assert!(error.contains("wrong input size"));
    }

    #[test]
    fn rejects_non_pe_input_before_hash_check() {
        let source = vec![0; SOURCE_SIZE];
        let error = prepare_patch(&source).expect_err("non-PE input must fail");
        assert!(error.contains("not an MZ executable"));
    }
}
