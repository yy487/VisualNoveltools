use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:X}", Sha256::digest(bytes))
}

pub fn hex_encode(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        let _ = write!(output, "{byte:02X}");
    }
    output
}

pub fn validate_relative_path(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        bail!(
            "manifest path must be a non-empty relative path: {}",
            path.display()
        );
    }
    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            bail!(
                "manifest path contains an unsafe component: {}",
                path.display()
            );
        }
    }
    Ok(())
}

pub fn prepare_output_directory(output: &Path, overwrite: bool) -> Result<PathBuf> {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        bail!("output parent does not exist: {}", parent.display());
    }
    if output.exists() && !overwrite {
        bail!(
            "output already exists (use --overwrite): {}",
            output.display()
        );
    }
    let name = output
        .file_name()
        .context("output directory has no final component")?
        .to_string_lossy();
    let temporary = parent.join(format!(".{name}.tmp-{}", std::process::id()));
    if temporary.exists() {
        bail!("temporary output already exists: {}", temporary.display());
    }
    fs::create_dir(&temporary)
        .with_context(|| format!("failed to create {}", temporary.display()))?;
    Ok(temporary)
}

pub fn commit_output_directory(temporary: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if output.exists() {
        if !overwrite {
            bail!("output already exists: {}", output.display());
        }
        fs::remove_dir_all(output)
            .with_context(|| format!("failed to remove existing {}", output.display()))?;
    }
    fs::rename(temporary, output).with_context(|| {
        format!(
            "failed to move {} to {}",
            temporary.display(),
            output.display()
        )
    })
}

pub fn write_output_file(output: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        bail!("output parent does not exist: {}", parent.display());
    }
    if output.exists() && !overwrite {
        bail!(
            "output already exists (use --overwrite): {}",
            output.display()
        );
    }
    let name = output
        .file_name()
        .context("output file has no final component")?
        .to_string_lossy();
    let temporary = parent.join(format!(".{name}.tmp-{}", std::process::id()));
    if temporary.exists() {
        bail!("temporary output already exists: {}", temporary.display());
    }
    fs::write(&temporary, bytes)
        .with_context(|| format!("failed to write {}", temporary.display()))?;
    if output.exists() {
        fs::remove_file(output)
            .with_context(|| format!("failed to replace {}", output.display()))?;
    }
    fs::rename(&temporary, output).with_context(|| {
        format!(
            "failed to move {} to {}",
            temporary.display(),
            output.display()
        )
    })
}
