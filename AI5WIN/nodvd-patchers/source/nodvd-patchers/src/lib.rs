use sha2::{Digest, Sha256};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const PATCH_BYTES: &[u8] = &[0xB8, 0x01, 0x00, 0x00, 0x00];
const NO_DISC_FALLBACK: &[u8] = &[0x31, 0xC0, 0xC3, 0x90, 0x90, 0x90];
const MEDIA_INIT_STUB: &[u8] = &[
    0xC7, 0x81, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // media type = 0
    0xC7, 0x81, 0xD4, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, // bytes/sector = 2048
    0xC7, 0x81, 0xD8, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, // sectors/cluster = 1
    0xB8, 0x01, 0x00, 0x00, 0x00, 0xC3, 0x90, 0x90, 0x90, 0x90,
];

pub struct PatchSite {
    pub description: &'static str,
    pub offset: usize,
    pub expected: &'static [u8],
    pub replacement: &'static [u8],
}

pub struct PatchProfile {
    pub game: &'static str,
    pub expected_size: usize,
    pub expected_sha256: &'static str,
    pub sites: &'static [PatchSite],
}

const SHANGRILA_1_SITES: &[PatchSite] = &[
    PatchSite {
        description: "primary media initialization",
        offset: 0x3670,
        expected: &[
            0x55, 0x8B, 0xEC, 0x83, 0xE4, 0xF8, 0x6A, 0xFF, 0x68, 0x3B, 0x61, 0x49, 0x00, 0x64,
            0xA1, 0x00, 0x00, 0x00, 0x00, 0x50, 0x64, 0x89, 0x25, 0x00, 0x00, 0x00, 0x00, 0x81,
            0xEC, 0x38, 0x02, 0x00, 0x00, 0xA1, 0x20, 0x0D, 0x4B, 0x00, 0x53, 0x56,
        ],
        replacement: MEDIA_INIT_STUB,
    },
    PatchSite {
        description: "EDP3 ProtectCheck",
        offset: 0x3900,
        expected: &[
            0x6A, 0xFF, 0x68, 0x7B, 0x61, 0x49, 0x00, 0x64, 0xA1, 0x00, 0x00, 0x00, 0x00,
        ],
        replacement: &[
            0xB8, 0x01, 0x00, 0x00, 0x00, 0xC2, 0x04, 0x00, 0x90, 0x90, 0x90, 0x90, 0x90,
        ],
    },
    PatchSite {
        description: "startup disc validation",
        offset: 0x3D77,
        expected: &[0xE8, 0xE4, 0x72, 0x00, 0x00],
        replacement: PATCH_BYTES,
    },
    PatchSite {
        description: "disc resource fallback",
        offset: 0xAE30,
        expected: &[0x81, 0xEC, 0x04, 0x04, 0x00, 0x00],
        replacement: NO_DISC_FALLBACK,
    },
    PatchSite {
        description: "secondary media initialization",
        offset: 0x35720,
        expected: &[
            0x55, 0x8B, 0xEC, 0x83, 0xE4, 0xF8, 0x6A, 0xFF, 0x68, 0x5B, 0x75, 0x49, 0x00, 0x64,
            0xA1, 0x00, 0x00, 0x00, 0x00, 0x50, 0x64, 0x89, 0x25, 0x00, 0x00, 0x00, 0x00, 0x81,
            0xEC, 0x40, 0x02, 0x00, 0x00, 0xA1, 0x20, 0x0D, 0x4B, 0x00, 0x53, 0x56,
        ],
        replacement: MEDIA_INIT_STUB,
    },
];

const SHANGRILA_2_SITES: &[PatchSite] = &[
    PatchSite {
        description: "primary media initialization",
        offset: 0x40A0,
        expected: &[
            0x55, 0x8B, 0xEC, 0x83, 0xE4, 0xF8, 0x6A, 0xFF, 0x68, 0xBB, 0x5A, 0x4A, 0x00, 0x64,
            0xA1, 0x00, 0x00, 0x00, 0x00, 0x50, 0x64, 0x89, 0x25, 0x00, 0x00, 0x00, 0x00, 0x81,
            0xEC, 0x38, 0x02, 0x00, 0x00, 0xA1, 0xAC, 0x30, 0x4C, 0x00, 0x53, 0x56,
        ],
        replacement: MEDIA_INIT_STUB,
    },
    PatchSite {
        description: "startup disc validation",
        offset: 0x4627,
        expected: &[0xE8, 0xF4, 0x73, 0x00, 0x00],
        replacement: PATCH_BYTES,
    },
    PatchSite {
        description: "disc resource fallback",
        offset: 0xB7F0,
        expected: &[0x81, 0xEC, 0x04, 0x04, 0x00, 0x00],
        replacement: NO_DISC_FALLBACK,
    },
    PatchSite {
        description: "secondary media initialization",
        offset: 0x3AEF0,
        expected: &[
            0x55, 0x8B, 0xEC, 0x83, 0xE4, 0xF8, 0x6A, 0xFF, 0x68, 0x6B, 0x6F, 0x4A, 0x00, 0x64,
            0xA1, 0x00, 0x00, 0x00, 0x00, 0x50, 0x64, 0x89, 0x25, 0x00, 0x00, 0x00, 0x00, 0x81,
            0xEC, 0x40, 0x02, 0x00, 0x00, 0xA1, 0xAC, 0x30, 0x4C, 0x00, 0x53, 0x56,
        ],
        replacement: MEDIA_INIT_STUB,
    },
];

pub const SHANGRILA_1: PatchProfile = PatchProfile {
    game: "Shangri-La 1",
    expected_size: 774_144,
    expected_sha256: "3021b7d7786dfb374b1e0a2c8a0e8c59ac14a65bf2c758e6d9f0d66f900c8401",
    sites: SHANGRILA_1_SITES,
};

pub const SHANGRILA_2: PatchProfile = PatchProfile {
    game: "Shangri-La 2",
    expected_size: 864_256,
    expected_sha256: "256a7ad85a2356c880c63795fadc036334ede53f73bd92c663b8e7d64ce22d87",
    sites: SHANGRILA_2_SITES,
};

#[derive(Debug)]
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

fn prepare_patch(profile: &PatchProfile, source: &[u8]) -> Result<(Vec<u8>, PatchReport), String> {
    if source.len() != profile.expected_size {
        return Err(format!(
            "wrong input size for {}: expected {} bytes, found {} bytes",
            profile.game,
            profile.expected_size,
            source.len()
        ));
    }

    let source_sha256 = sha256_hex(source);
    if source_sha256 != profile.expected_sha256 {
        return Err(format!(
            "wrong input SHA-256 for {}: expected {}, found {}",
            profile.game, profile.expected_sha256, source_sha256
        ));
    }

    if profile.sites.is_empty() {
        return Err("patch profile has no sites".to_owned());
    }
    for (index, site) in profile.sites.iter().enumerate() {
        if site.expected.is_empty() || site.expected.len() != site.replacement.len() {
            return Err(format!(
                "invalid fixed-length patch site at 0x{:X}",
                site.offset
            ));
        }
        let end = site
            .offset
            .checked_add(site.expected.len())
            .ok_or_else(|| "patch range overflow".to_owned())?;
        if end > source.len() {
            return Err(format!(
                "patch range at 0x{:X} is outside the input",
                site.offset
            ));
        }
        for other in &profile.sites[index + 1..] {
            let other_end = other
                .offset
                .checked_add(other.expected.len())
                .ok_or_else(|| "patch range overflow".to_owned())?;
            if site.offset < other_end && other.offset < end {
                return Err(format!(
                    "overlapping patch sites at 0x{:X} and 0x{:X}",
                    site.offset, other.offset
                ));
            }
        }
        let actual = &source[site.offset..end];
        if actual != site.expected {
            return Err(format!(
                "original bytes do not match at 0x{:X} ({}): expected [{}], found [{}]",
                site.offset,
                site.description,
                hex_bytes(site.expected),
                hex_bytes(actual)
            ));
        }
    }

    let mut output = source.to_vec();
    for site in profile.sites {
        let end = site.offset + site.replacement.len();
        output[site.offset..end].copy_from_slice(site.replacement);
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
        },
    ))
}

fn default_input() -> Result<PathBuf, String> {
    let executable = env::current_exe()
        .map_err(|error| format!("cannot locate the patcher executable: {error}"))?;
    let directory = executable
        .parent()
        .ok_or_else(|| "patcher executable has no parent directory".to_owned())?;
    Ok(directory.join("AI5WIN.exe"))
}

fn default_output(input: &Path) -> Result<PathBuf, String> {
    let stem = input
        .file_stem()
        .ok_or_else(|| format!("input has no file stem: {}", input.display()))?;
    let mut name = OsString::from(stem);
    name.push("_nodvd.exe");
    Ok(input.with_file_name(name))
}

fn parse_args() -> Result<Option<(PathBuf, PathBuf)>, String> {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut args = env::args_os().skip(1);

    while let Some(arg) = args.next() {
        if arg == OsStr::new("-h") || arg == OsStr::new("--help") {
            return Ok(None);
        }
        if arg == OsStr::new("-o") || arg == OsStr::new("--output") {
            let value = args
                .next()
                .ok_or_else(|| format!("{} requires a path", arg.to_string_lossy()))?;
            if output.replace(PathBuf::from(value)).is_some() {
                return Err("output was specified more than once".to_owned());
            }
            continue;
        }
        if arg.to_string_lossy().starts_with('-') {
            return Err(format!("unknown option: {}", arg.to_string_lossy()));
        }
        if input.replace(PathBuf::from(arg)).is_some() {
            return Err("only one input EXE is supported".to_owned());
        }
    }

    let input = match input {
        Some(path) => path,
        None => default_input()?,
    };
    let output = match output {
        Some(path) => path,
        None => default_output(&input)?,
    };
    Ok(Some((input, output)))
}

fn print_help(profile: &PatchProfile) {
    println!("AI5WIN NoDVD patcher for {}", profile.game);
    println!();
    println!("Usage:");
    println!("  PATCHER.exe [INPUT] [-o|--output OUTPUT]");
    println!();
    println!("With no INPUT, the patcher reads AI5WIN.exe beside itself.");
    println!("The default output is AI5WIN_nodvd.exe beside the input.");
    println!("Existing outputs are never overwritten.");
    println!("Drag-and-drop of one AI5WIN.exe is supported.");
}

fn write_verified_output(
    profile: &PatchProfile,
    output_path: &Path,
    output: &[u8],
) -> Result<(), String> {
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
    let replacements_match = profile.sites.iter().all(|site| {
        let end = site.offset + site.replacement.len();
        written[site.offset..end] == *site.replacement
    });
    if written != output || !replacements_match {
        let _ = fs::remove_file(output_path);
        return Err(format!(
            "output verification failed; incomplete file was removed: {}",
            output_path.display()
        ));
    }
    Ok(())
}

pub fn run_cli(profile: &PatchProfile) -> Result<(), String> {
    let Some((input_path, output_path)) = parse_args()? else {
        print_help(profile);
        return Ok(());
    };

    if input_path == output_path {
        return Err("input and output paths must differ".to_owned());
    }
    if output_path.exists() {
        return Err(format!(
            "output already exists; refusing to overwrite: {}",
            output_path.display()
        ));
    }

    let source = fs::read(&input_path)
        .map_err(|error| format!("cannot read {}: {error}", input_path.display()))?;
    let (output, report) = prepare_patch(profile, &source)?;
    write_verified_output(profile, &output_path, &output)?;

    println!("[target] game={}", profile.game);
    println!("[patch] input={}", input_path.display());
    println!("[patch] source_sha256={}", report.source_sha256);
    println!("[patch] sites={}", profile.sites.len());
    for site in profile.sites {
        println!(
            "[patch] site={} offset=0x{:X} original=[{}] replacement=[{}]",
            site.description,
            site.offset,
            hex_bytes(site.expected),
            hex_bytes(site.replacement)
        );
    }
    println!("[patch] changed_bytes={}", report.changed_bytes);
    println!("[patch] output_sha256={}", report.output_sha256);
    println!("[patch] output={}", output_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SITES: &[PatchSite] = &[
        PatchSite {
            description: "first",
            offset: 0,
            expected: b"a",
            replacement: b"x",
        },
        PatchSite {
            description: "second",
            offset: 2,
            expected: b"c",
            replacement: b"z",
        },
    ];

    const TEST_PROFILE: PatchProfile = PatchProfile {
        game: "test",
        expected_size: 3,
        expected_sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        sites: TEST_SITES,
    };

    #[test]
    fn applies_only_the_declared_patch() {
        let (patched, report) = prepare_patch(&TEST_PROFILE, b"abc").expect("valid patch");
        assert_eq!(patched, b"xbz");
        assert_eq!(report.changed_bytes, 2);
        assert_eq!(report.source_sha256, TEST_PROFILE.expected_sha256);
    }

    #[test]
    fn rejects_wrong_hash() {
        let error = prepare_patch(&TEST_PROFILE, b"abd").expect_err("hash must fail");
        assert!(error.contains("wrong input SHA-256"));
    }

    #[test]
    fn rejects_wrong_context_even_with_matching_hash() {
        const BAD_SITES: &[PatchSite] = &[PatchSite {
            description: "bad",
            offset: 1,
            expected: b"d",
            replacement: b"x",
        }];
        let profile = PatchProfile {
            sites: BAD_SITES,
            ..TEST_PROFILE
        };
        let error = prepare_patch(&profile, b"abc").expect_err("context must fail");
        assert!(error.contains("original bytes do not match"));
    }

    #[test]
    fn rejects_overlapping_sites() {
        const OVERLAPPING_SITES: &[PatchSite] = &[
            PatchSite {
                description: "first",
                offset: 0,
                expected: b"ab",
                replacement: b"xy",
            },
            PatchSite {
                description: "second",
                offset: 1,
                expected: b"bc",
                replacement: b"yz",
            },
        ];
        let profile = PatchProfile {
            sites: OVERLAPPING_SITES,
            ..TEST_PROFILE
        };
        let error = prepare_patch(&profile, b"abc").expect_err("overlap must fail");
        assert!(error.contains("overlapping patch sites"));
    }

    #[test]
    fn default_output_keeps_the_parent_directory() {
        let output = default_output(Path::new(r"C:\game dir\AI5WIN.exe")).expect("output name");
        assert_eq!(output, PathBuf::from(r"C:\game dir\AI5WIN_nodvd.exe"));
    }
}
