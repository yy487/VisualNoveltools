use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub const SOURCE_SIZE: usize = 262_144;
pub const SOURCE_SHA256: &str = "b9334fca270c138dc375e41ff708dab1c788237fe8cebf9ebd5161a2ae630f16";
pub const OLD_CHARSET: u8 = 0x80;
pub const NEW_CHARSET: u8 = 0x86;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatchSite {
    pub instruction_offset: usize,
    pub value_offset: usize,
    pub expected: &'static [u8],
}

// These are the seven font charset arguments in this exact CHS executable.
pub const PATCH_SITES: [PatchSite; 7] = [
    PatchSite {
        instruction_offset: 0xA97F,
        value_offset: 0xA980,
        expected: &[0x68, OLD_CHARSET, 0x00, 0x00, 0x00],
    },
    PatchSite {
        instruction_offset: 0xAB61,
        value_offset: 0xAB65,
        expected: &[0xC6, 0x44, 0x24, 0x2F, OLD_CHARSET],
    },
    PatchSite {
        instruction_offset: 0xAC16,
        value_offset: 0xAC1A,
        expected: &[0xC6, 0x44, 0x24, 0x3B, OLD_CHARSET],
    },
    PatchSite {
        instruction_offset: 0xAF66,
        value_offset: 0xAF6A,
        expected: &[0xC6, 0x44, 0x24, 0x3F, OLD_CHARSET],
    },
    PatchSite {
        instruction_offset: 0xB032,
        value_offset: 0xB033,
        expected: &[0x68, OLD_CHARSET, 0x00, 0x00, 0x00],
    },
    PatchSite {
        instruction_offset: 0xB10E,
        value_offset: 0xB10F,
        expected: &[0x68, OLD_CHARSET, 0x00, 0x00, 0x00],
    },
    PatchSite {
        instruction_offset: 0x1758A,
        value_offset: 0x1758B,
        expected: &[0x68, OLD_CHARSET, 0x00, 0x00, 0x00],
    },
];

#[derive(Debug, PartialEq, Eq)]
pub struct PatchReport {
    pub source_sha256: String,
    pub output_sha256: String,
    pub changed_bytes: usize,
    pub patched_sites: usize,
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

    let mut output = source.to_vec();
    for site in PATCH_SITES {
        let end = site
            .instruction_offset
            .checked_add(site.expected.len())
            .ok_or_else(|| format!("patch window overflow at 0x{:X}", site.instruction_offset))?;
        let actual = source.get(site.instruction_offset..end).ok_or_else(|| {
            format!(
                "patch window is outside the input at 0x{:X}",
                site.instruction_offset
            )
        })?;
        if actual != site.expected {
            return Err(format!(
                "font charset bytes do not match at 0x{:X}: expected [{}], found [{}]",
                site.instruction_offset,
                hex_bytes(site.expected),
                hex_bytes(actual)
            ));
        }
        if output[site.value_offset] != OLD_CHARSET {
            return Err(format!(
                "font charset byte is not 0x{OLD_CHARSET:02X} at 0x{:X}",
                site.value_offset
            ));
        }
        output[site.value_offset] = NEW_CHARSET;
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
            patched_sites: PATCH_SITES.len(),
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
    PATCH_SITES
        .iter()
        .map(|site| format!("0x{:X}->0x{:X}", site.value_offset, NEW_CHARSET))
        .collect::<Vec<_>>()
        .join(",")
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
    fn patches_only_the_seven_font_charset_bytes() {
        let source = matching_source();
        let (output, report) = prepare_patch(&source).expect("sample must patch");
        assert_eq!(output.len(), source.len());
        assert_eq!(report.patched_sites, PATCH_SITES.len());
        assert_eq!(report.changed_bytes, PATCH_SITES.len());
        assert_eq!(report.output_sha256, sha256_hex(&output));
        assert_eq!(output[0xC747], 0xEB, "preserve the existing NoDVD branch");
        for site in PATCH_SITES {
            assert_eq!(output[site.value_offset], NEW_CHARSET);
        }
        assert!(source
            .iter()
            .zip(&output)
            .enumerate()
            .all(|(offset, (before, after))| {
                PATCH_SITES.iter().any(|site| site.value_offset == offset) || before == after
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
    fn rejects_changed_instruction_bytes() {
        let mut source = matching_source();
        source[0xA97F] ^= 1;
        let error = prepare_patch(&source).expect_err("changed patch site must fail");
        assert!(error.contains("wrong input SHA-256"));
    }
}
