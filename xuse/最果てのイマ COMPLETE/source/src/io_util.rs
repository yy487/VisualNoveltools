use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::ToolResult;

pub fn read_u16(bytes: &[u8], offset: usize, context: &str) -> ToolResult<u16> {
    let slice = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("{context}: truncated u16 at 0x{offset:X}"))?;
    Ok(u16::from_le_bytes(slice.try_into().unwrap()))
}

pub fn read_u32(bytes: &[u8], offset: usize, context: &str) -> ToolResult<u32> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("{context}: truncated u32 at 0x{offset:X}"))?;
    Ok(u32::from_le_bytes(slice.try_into().unwrap()))
}

pub fn put_u16(bytes: &mut [u8], offset: usize, value: u16, context: &str) -> ToolResult<()> {
    let target = bytes
        .get_mut(offset..offset + 2)
        .ok_or_else(|| format!("{context}: cannot write u16 at 0x{offset:X}"))?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

pub fn put_u32(bytes: &mut [u8], offset: usize, value: u32, context: &str) -> ToolResult<()> {
    let target = bytes
        .get_mut(offset..offset + 4)
        .ok_or_else(|| format!("{context}: cannot write u32 at 0x{offset:X}"))?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

pub fn checked_add(a: usize, b: usize, context: &str) -> ToolResult<usize> {
    a.checked_add(b)
        .ok_or_else(|| format!("{context}: integer overflow"))
}

pub fn checked_mul(a: usize, b: usize, context: &str) -> ToolResult<usize> {
    a.checked_mul(b)
        .ok_or_else(|| format!("{context}: integer overflow"))
}

pub fn normalize_relative(path: &str) -> ToolResult<PathBuf> {
    let normalized = path.replace('/', "\\");
    let candidate = Path::new(&normalized);
    if candidate.is_absolute() {
        return Err(format!("_file must be relative, got {path:?}"));
    }
    let mut out = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::Normal(value) => out.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return Err(format!("unsafe relative path {path:?}"));
            }
        }
    }
    if out.as_os_str().is_empty() {
        return Err("_file cannot be empty".to_string());
    }
    Ok(out)
}

pub fn relative_string(root: &Path, path: &Path) -> ToolResult<String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("{} is outside {}", path.display(), root.display()))?;
    Ok(relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

pub fn list_files_recursive(root: &Path) -> ToolResult<Vec<PathBuf>> {
    if !root.is_dir() {
        return Err(format!("directory not found: {}", root.display()));
    }
    let mut out = Vec::new();
    visit(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn visit(path: &Path, out: &mut Vec<PathBuf>) -> ToolResult<()> {
    let entries = fs::read_dir(path)
        .map_err(|error| format!("cannot read directory {}: {error}", path.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("cannot inspect {}: {error}", entry.path().display()))?;
        if file_type.is_dir() {
            visit(&entry.path(), out)?;
        } else if file_type.is_file() {
            out.push(entry.path());
        }
    }
    Ok(())
}

pub fn copy_tree(source: &Path, output: &Path) -> ToolResult<u64> {
    let files = list_files_recursive(source)?;
    fs::create_dir_all(output)
        .map_err(|error| format!("cannot create {}: {error}", output.display()))?;
    let mut count = 0u64;
    for file in files {
        let relative = file
            .strip_prefix(source)
            .map_err(|_| format!("{} escaped source root", file.display()))?;
        let target = output.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        fs::copy(&file, &target).map_err(|error| {
            format!(
                "cannot copy {} to {}: {error}",
                file.display(),
                target.display()
            )
        })?;
        count += 1;
    }
    Ok(count)
}

pub fn prepare_output(path: &Path, overwrite: bool) -> ToolResult<()> {
    let resolved = absolute_nonexistent(path)?;
    if resolved.parent().is_none() || resolved.file_name().is_none() {
        return Err(format!(
            "refusing to use a filesystem root as output: {}",
            path.display()
        ));
    }
    let current = std::env::current_dir()
        .map_err(|error| format!("cannot resolve current directory: {error}"))?
        .canonicalize()
        .map_err(|error| format!("cannot resolve current directory: {error}"))?;
    if resolved == current {
        return Err(format!(
            "refusing to replace the current working directory: {}",
            path.display()
        ));
    }
    if path.exists() {
        if !overwrite {
            return Err(format!(
                "output already exists: {}; pass --overwrite to replace it",
                path.display()
            ));
        }
        if path.is_dir() {
            fs::remove_dir_all(path)
                .map_err(|error| format!("cannot remove {}: {error}", path.display()))?;
        } else {
            fs::remove_file(path)
                .map_err(|error| format!("cannot remove {}: {error}", path.display()))?;
        }
    }
    Ok(())
}

pub fn reject_output_overlap(source: &Path, other: Option<&Path>, output: &Path) -> ToolResult<()> {
    let source = source
        .canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", source.display()))?;
    let other = other
        .map(|path| {
            path.canonicalize()
                .map_err(|error| format!("cannot resolve {}: {error}", path.display()))
        })
        .transpose()?;
    let output_absolute = absolute_nonexistent(output)?;
    if output_absolute == source
        || output_absolute.starts_with(&source)
        || source.starts_with(&output_absolute)
    {
        return Err(format!(
            "output {} must not overlap the source directory",
            output.display()
        ));
    }
    if let Some(other) = other {
        if source == other || source.starts_with(&other) || other.starts_with(&source) {
            return Err("source and translation directories must not overlap".to_string());
        }
        if output_absolute == other
            || output_absolute.starts_with(&other)
            || other.starts_with(&output_absolute)
        {
            return Err(format!(
                "output {} must not overlap the translation directory",
                output.display()
            ));
        }
    }
    Ok(())
}

fn absolute_nonexistent(path: &Path) -> ToolResult<PathBuf> {
    if path.exists() {
        return path
            .canonicalize()
            .map_err(|error| format!("cannot resolve {}: {error}", path.display()));
    }
    let mut cursor = path;
    let mut suffix = Vec::new();
    while !cursor.exists() {
        let name = cursor
            .file_name()
            .ok_or_else(|| format!("invalid output path: {}", path.display()))?;
        suffix.push(name.to_os_string());
        cursor = cursor.parent().unwrap_or_else(|| Path::new("."));
    }
    let mut resolved = cursor
        .canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", cursor.display()))?;
    for name in suffix.into_iter().rev() {
        resolved.push(name);
    }
    Ok(resolved)
}
