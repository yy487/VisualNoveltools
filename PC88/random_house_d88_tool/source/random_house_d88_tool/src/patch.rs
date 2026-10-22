use crate::basic::BasicProgram;
use crate::d88::D88Image;
use crate::renderer::{resident_renderer, EXPECTED_SHA256};
use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const BASIC_ATTRIBUTE: u8 = 0x80;
const BOOT_PROGRAM: &str = "TITL";

#[derive(Debug, Clone)]
pub struct RoundtripReport {
    pub files: usize,
    pub basic_files: usize,
    pub byte_exact: bool,
    pub output_bytes: usize,
    pub output_sha256: String,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BootstrapPatchReport {
    pub files: usize,
    pub basic_files: usize,
    pub clear_patches: usize,
    pub changed_files: usize,
    pub renderer_bytes: usize,
    pub renderer_sha256: String,
    pub payload_line_start: u16,
    pub title_old_bytes: usize,
    pub title_new_bytes: usize,
    pub title_program_bytes: usize,
    pub comment_bytes_removed: usize,
    pub output_bytes: usize,
    pub output_sha256: String,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ReservePatchReport {
    pub files: usize,
    pub basic_files: usize,
    pub clear_patches: usize,
    pub changed_files: usize,
    pub output_bytes: usize,
    pub output_sha256: String,
    pub output: PathBuf,
}

pub type CommentProbeReport = ReservePatchReport;

pub fn roundtrip_disk(source: &Path, output: &Path, overwrite: bool) -> Result<RoundtripReport> {
    reject_source_overwrite(source, output)?;
    let source_bytes = fs::read(source)
        .with_context(|| format!("failed to read source D88 {}", source.display()))?;
    let image = D88Image::parse(source_bytes.clone())
        .with_context(|| format!("failed to parse source D88 {}", source.display()))?;
    let mut basic_files = 0usize;
    for file in &image.files {
        if file.attribute != BASIC_ATTRIBUTE {
            continue;
        }
        basic_files += 1;
        let program = BasicProgram::parse(&file.data)
            .with_context(|| format!("failed to parse tokenized BASIC file {}", file.name))?;
        if program.byte_exact_rebuild()? != file.data {
            bail!(
                "tokenized BASIC no-change rebuild mismatch for {}",
                file.name
            );
        }
    }
    let rebuilt = image.rebuild(&HashMap::new())?;
    let byte_exact = rebuilt.bytes == source_bytes;
    if !byte_exact {
        bail!("D88 no-change structural rebuild is not byte-exact");
    }
    write_output(output, &rebuilt.bytes, overwrite)?;
    Ok(RoundtripReport {
        files: image.files.len(),
        basic_files,
        byte_exact,
        output_bytes: rebuilt.bytes.len(),
        output_sha256: sha256_hex(&rebuilt.bytes),
        output: output.to_path_buf(),
    })
}

pub fn patch_bootstrap(
    source: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<BootstrapPatchReport> {
    reject_source_overwrite(source, output)?;
    let source_bytes = fs::read(source)
        .with_context(|| format!("failed to read source D88 {}", source.display()))?;
    let image = D88Image::parse(source_bytes)
        .with_context(|| format!("failed to parse source D88 {}", source.display()))?;
    let renderer = resident_renderer()?;
    let mut replacements = HashMap::new();
    let mut basic_files = 0usize;
    let mut clear_patches = 0usize;
    let mut payload_line_start = None;
    let mut title_program_bytes = None;
    let mut title_old_bytes = None;
    let mut title_new_bytes = None;
    let mut comment_bytes_removed = 0usize;
    let disk_has_no_free_clusters = image.free_cluster_count() == 0;

    for file in &image.files {
        if file.attribute != BASIC_ATTRIBUTE {
            continue;
        }
        basic_files += 1;
        let mut program = BasicProgram::parse(&file.data)
            .with_context(|| format!("failed to parse tokenized BASIC file {}", file.name))?;
        if program.byte_exact_rebuild()? != file.data {
            bail!(
                "tokenized BASIC no-change rebuild mismatch for {}",
                file.name
            );
        }
        program
            .patch_memory_ceiling()
            .with_context(|| format!("failed to reserve resident RAM in {}", file.name))?;
        clear_patches += 1;

        let bytes = if file.name.eq_ignore_ascii_case(BOOT_PROGRAM) {
            title_old_bytes = Some(file.data.len());
            let capacity = disk_has_no_free_clusters.then_some(file.clusters.len() * 8 * 256);
            let bootstrap = program
                .install_bootstrap(&renderer, capacity)
                .context("failed to insert resident renderer bootstrap into TITL")?;
            payload_line_start = Some(bootstrap.payload_line_start);
            title_program_bytes = Some(bootstrap.program_bytes);
            title_new_bytes = Some(bootstrap.bytes.len());
            comment_bytes_removed = bootstrap.comment_bytes_removed;
            bootstrap.bytes
        } else {
            program.byte_exact_rebuild()?
        };
        replacements.insert(file.name.clone(), bytes);
    }
    if basic_files != 15 || clear_patches != 15 {
        bail!(
            "expected 15 BASIC files and CLEAR patches, got basic={basic_files}, clear={clear_patches}"
        );
    }
    let payload_line_start = payload_line_start.context("TITL bootstrap was not inserted")?;
    let rebuilt = image.rebuild(&replacements)?;

    let verified = D88Image::parse(rebuilt.bytes.clone())?;
    let verified_title = verified.file(BOOT_PROGRAM)?;
    let verified_title_program = BasicProgram::parse(&verified_title.data)?;
    if verified_title_program.embedded_payload()?.as_deref() != Some(renderer.as_slice()) {
        bail!("rebuilt TITL does not contain the exact resident renderer payload");
    }
    for file in &verified.files {
        if file.attribute != BASIC_ATTRIBUTE {
            continue;
        }
        let program = BasicProgram::parse(&file.data)
            .with_context(|| format!("rebuilt BASIC parse failed for {}", file.name))?;
        let rebuilt_again = program.byte_exact_rebuild()?;
        if rebuilt_again != file.data {
            bail!("rebuilt BASIC roundtrip mismatch for {}", file.name);
        }
    }

    write_output(output, &rebuilt.bytes, overwrite)?;
    Ok(BootstrapPatchReport {
        files: image.files.len(),
        basic_files,
        clear_patches,
        changed_files: rebuilt.changed_files,
        renderer_bytes: renderer.len(),
        renderer_sha256: EXPECTED_SHA256.to_owned(),
        payload_line_start,
        title_old_bytes: title_old_bytes.context("missing original TITL size")?,
        title_new_bytes: title_new_bytes.context("missing rebuilt TITL size")?,
        title_program_bytes: title_program_bytes.context("missing rebuilt TITL program size")?,
        comment_bytes_removed,
        output_bytes: rebuilt.bytes.len(),
        output_sha256: sha256_hex(&rebuilt.bytes),
        output: output.to_path_buf(),
    })
}

pub fn patch_reserve_only(
    source: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<ReservePatchReport> {
    reject_source_overwrite(source, output)?;
    let source_bytes = fs::read(source)
        .with_context(|| format!("failed to read source D88 {}", source.display()))?;
    let image = D88Image::parse(source_bytes)
        .with_context(|| format!("failed to parse source D88 {}", source.display()))?;
    let mut replacements = HashMap::new();
    let mut basic_files = 0usize;
    let mut clear_patches = 0usize;
    for file in &image.files {
        if file.attribute != BASIC_ATTRIBUTE {
            continue;
        }
        basic_files += 1;
        let mut program = BasicProgram::parse(&file.data)
            .with_context(|| format!("failed to parse tokenized BASIC file {}", file.name))?;
        if program.byte_exact_rebuild()? != file.data {
            bail!(
                "tokenized BASIC no-change rebuild mismatch for {}",
                file.name
            );
        }
        program
            .patch_memory_ceiling()
            .with_context(|| format!("failed to reserve resident RAM in {}", file.name))?;
        clear_patches += 1;
        replacements.insert(file.name.clone(), program.byte_exact_rebuild()?);
    }
    if basic_files != 15 || clear_patches != 15 {
        bail!(
            "expected 15 BASIC files and CLEAR patches, got basic={basic_files}, clear={clear_patches}"
        );
    }
    let rebuilt = image.rebuild(&replacements)?;
    write_output(output, &rebuilt.bytes, overwrite)?;
    Ok(ReservePatchReport {
        files: image.files.len(),
        basic_files,
        clear_patches,
        changed_files: rebuilt.changed_files,
        output_bytes: rebuilt.bytes.len(),
        output_sha256: sha256_hex(&rebuilt.bytes),
        output: output.to_path_buf(),
    })
}

pub fn patch_comment_probe(
    source: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<CommentProbeReport> {
    reject_source_overwrite(source, output)?;
    let source_bytes = fs::read(source)
        .with_context(|| format!("failed to read source D88 {}", source.display()))?;
    let image = D88Image::parse(source_bytes)
        .with_context(|| format!("failed to parse source D88 {}", source.display()))?;
    let mut replacements = HashMap::new();
    let mut basic_files = 0usize;
    let mut clear_patches = 0usize;
    for file in &image.files {
        if file.attribute != BASIC_ATTRIBUTE {
            continue;
        }
        basic_files += 1;
        let mut program = BasicProgram::parse(&file.data)
            .with_context(|| format!("failed to parse tokenized BASIC file {}", file.name))?;
        program
            .patch_memory_ceiling()
            .with_context(|| format!("failed to reserve resident RAM in {}", file.name))?;
        clear_patches += 1;
        let bytes = if file.name.eq_ignore_ascii_case(BOOT_PROGRAM) {
            program.install_comment_probe()?
        } else {
            program.byte_exact_rebuild()?
        };
        replacements.insert(file.name.clone(), bytes);
    }
    let rebuilt = image.rebuild(&replacements)?;
    write_output(output, &rebuilt.bytes, overwrite)?;
    Ok(CommentProbeReport {
        files: image.files.len(),
        basic_files,
        clear_patches,
        changed_files: rebuilt.changed_files,
        output_bytes: rebuilt.bytes.len(),
        output_sha256: sha256_hex(&rebuilt.bytes),
        output: output.to_path_buf(),
    })
}

pub fn patch_payload_probe(
    source: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<CommentProbeReport> {
    reject_source_overwrite(source, output)?;
    let source_bytes = fs::read(source)
        .with_context(|| format!("failed to read source D88 {}", source.display()))?;
    let image = D88Image::parse(source_bytes)
        .with_context(|| format!("failed to parse source D88 {}", source.display()))?;
    let renderer = resident_renderer()?;
    let mut replacements = HashMap::new();
    let mut basic_files = 0usize;
    let mut clear_patches = 0usize;
    for file in &image.files {
        if file.attribute != BASIC_ATTRIBUTE {
            continue;
        }
        basic_files += 1;
        let mut program = BasicProgram::parse(&file.data)
            .with_context(|| format!("failed to parse tokenized BASIC file {}", file.name))?;
        program.patch_memory_ceiling()?;
        clear_patches += 1;
        let bytes = if file.name.eq_ignore_ascii_case(BOOT_PROGRAM) {
            program.install_payload_probe(&renderer)?
        } else {
            program.byte_exact_rebuild()?
        };
        replacements.insert(file.name.clone(), bytes);
    }
    let rebuilt = image.rebuild(&replacements)?;
    write_output(output, &rebuilt.bytes, overwrite)?;
    Ok(CommentProbeReport {
        files: image.files.len(),
        basic_files,
        clear_patches,
        changed_files: rebuilt.changed_files,
        output_bytes: rebuilt.bytes.len(),
        output_sha256: sha256_hex(&rebuilt.bytes),
        output: output.to_path_buf(),
    })
}

fn write_output(path: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        bail!(
            "output already exists: {}; pass --overwrite to replace it",
            path.display()
        );
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        bail!("output parent is not a directory: {}", parent.display());
    }
    fs::write(path, bytes).with_context(|| format!("failed to write output {}", path.display()))
}

fn reject_source_overwrite(source: &Path, output: &Path) -> Result<()> {
    let source = source
        .canonicalize()
        .with_context(|| format!("failed to resolve source {}", source.display()))?;
    let output = normalize_output(output)?;
    if source == output {
        bail!("output must not overwrite the source D88");
    }
    Ok(())
}

fn normalize_output(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return path
            .canonicalize()
            .with_context(|| format!("failed to resolve output {}", path.display()));
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = parent
        .canonicalize()
        .with_context(|| format!("failed to resolve output parent {}", parent.display()))?;
    let name = path.file_name().context("output path has no filename")?;
    Ok(parent.join(name))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:X}", Sha256::digest(bytes))
}
