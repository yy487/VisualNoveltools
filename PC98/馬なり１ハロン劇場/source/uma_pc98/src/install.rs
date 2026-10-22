use crate::Result;
use encoding_rs::SHIFT_JIS;
use std::{collections::BTreeMap, path::PathBuf};

fn dos_name(name: &str) -> Result<String> {
    let name = name.trim().to_ascii_uppercase();
    let parts: Vec<_> = name.split('.').collect();
    if parts.len() != 2
        || parts[0].is_empty()
        || parts[0].len() > 8
        || parts[1].is_empty()
        || parts[1].len() > 3
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.')
    {
        return Err(format!("安装清单中不是安全的 DOS 8.3 文件名: {name}"));
    }
    Ok(name)
}

/// Follow the game's installer copy/rename list, including all initial save slots.
pub fn build(native: &BTreeMap<String, Vec<u8>>) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    let recipe = native
        .get("01_DISK/FILELIST.INF")
        .ok_or("缺少原版 FILELIST.INF 安装清单")?;
    let text = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(recipe)
        .ok_or("安装清单不是有效 CP932")?;
    let mut disk = None;
    let mut installed = BTreeMap::new();
    for line in text
        .split('\x1a')
        .next()
        .unwrap_or("")
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if line == ":----------" {
            break;
        }
        if line.starts_with(':') {
            disk = match line.chars().last() {
                Some('１') => Some(1),
                Some('２') => Some(2),
                Some('３') => Some(3),
                Some('４') => Some(4),
                _ => return Err(format!("未知安装卷标签: {line}")),
            };
            continue;
        }
        let disk = disk.ok_or("安装文件出现在卷标签之前")?;
        let (source, destination) = line.split_once(',').unwrap_or((line, line));
        let source = dos_name(source)?;
        let destination = dos_name(destination)?;
        let path = format!("{disk:02}_DISK/{source}");
        let data = native
            .get(&path)
            .ok_or_else(|| format!("安装清单要求的文件不存在: {path}"))?;
        let target = PathBuf::from("installed/MV/UMA").join(destination);
        if installed.insert(target, data.clone()).is_some() {
            return Err(format!("安装清单有重复目标: {line}"));
        }
    }
    if installed.is_empty() {
        return Err("安装清单没有文件".into());
    }
    Ok(installed)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> BTreeMap<String, Vec<u8>> {
        let (recipe, _, errors) = SHIFT_JIS.encode(":ディスク－１\r\nUMA0.EXE,UMA.EXE\r\n:ディスク－２\r\nPARAM.TMP\r\nPARAM.TMP,PARAM.0\r\n:----------\r\n\x1a");
        assert!(!errors);
        BTreeMap::from([
            ("01_DISK/FILELIST.INF".into(), recipe.into_owned()),
            ("01_DISK/UMA0.EXE".into(), b"launcher".to_vec()),
            ("02_DISK/PARAM.TMP".into(), b"save seed".to_vec()),
        ])
    }
    #[test]
    fn installer_renames_launcher_and_copies_save_seed() {
        let result = build(&fixture()).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(
            result[&PathBuf::from("installed/MV/UMA/UMA.EXE")],
            b"launcher"
        );
        assert_eq!(
            result[&PathBuf::from("installed/MV/UMA/PARAM.0")],
            b"save seed"
        );
        assert_eq!(
            result[&PathBuf::from("installed/MV/UMA/PARAM.TMP")],
            b"save seed"
        );
    }
    #[test]
    fn incomplete_or_unsafe_installation_is_rejected() {
        let mut native = fixture();
        native.remove("02_DISK/PARAM.TMP");
        assert!(build(&native).is_err());
        for name in ["../BAD.EXE", "SUB/X.EXE", "C:BAD.EXE", "LONGFILENAME.EXE"] {
            assert!(dos_name(name).is_err());
        }
    }
}
