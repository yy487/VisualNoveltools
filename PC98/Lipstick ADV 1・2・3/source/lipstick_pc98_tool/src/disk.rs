use crate::{Game, Result, diskx, dword, word};
use fivec_new::Archive;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageInfo {
    pub name: String,
    pub unit: String,
}

pub struct Image {
    pub info: ImageInfo,
    pub bytes: Vec<u8>,
}

pub struct Unit {
    pub id: String,
    pub image: usize,
    pub parent: Option<(String, String)>,
    pub view: Vec<u8>,
    pub original_container: Option<Vec<u8>>,
    pub removed_labels: Vec<(usize, Vec<u8>)>,
    pub files: BTreeMap<String, Vec<u8>>,
}

pub struct Disks {
    pub images: Vec<Image>,
    pub units: Vec<Unit>,
}

pub fn discover(source: &Path) -> Result<Vec<PathBuf>> {
    if source.is_file() {
        return Ok(vec![source.to_path_buf()]);
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(source).map_err(|e| format!("{}: {e}", source.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            return Err("原盘目录中有链接，请直接选择原始镜像目录".into());
        }
        if meta.is_file()
            && path
                .extension()
                .is_some_and(|s| s.eq_ignore_ascii_case("fdi"))
        {
            paths.push(path);
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err("目录内没有 FDI；请选择其中一部游戏的原盘目录".into());
    }
    Ok(paths)
}

fn label_view(bytes: &mut [u8], game: Game) -> Result<Vec<(usize, Vec<u8>)>> {
    if game != Game::Lipstick2 {
        return Ok(Vec::new());
    }
    let base = dword(bytes, 8)? as usize;
    let bps = usize::from(word(bytes, base + 11)?);
    let reserved = usize::from(word(bytes, base + 14)?);
    let fats = usize::from(*bytes.get(base + 16).ok_or("BPB 截断")?);
    let spf = usize::from(word(bytes, base + 22)?);
    let count = usize::from(word(bytes, base + 17)?);
    let root = base
        .checked_add((reserved + fats * spf) * bps)
        .ok_or("目录偏移溢出")?;
    let label = bytes.get(root..root + 32).ok_or("二代根目录截断")?.to_vec();
    if label[11] != 0x29 || !matches!(&label[..11], b"Lip.Adv.2 A" | b"Lip.Adv.2 B") {
        return Err("不是已确认的 Lipstick 2 A/B 盘卷标".into());
    }
    let mut removed = Vec::new();
    for p in (root + 32..root + count * 32).step_by(32) {
        let entry = bytes.get_mut(p..p + 32).ok_or("二代根目录越界")?;
        if entry[..11] == label[..11] {
            if entry[11] != 0x27 || entry[26..32] != [0; 6] {
                return Err("同名卷标伪文件具有未知属性／内容".into());
            }
            removed.push((p, entry.to_vec()));
            entry[0] = 0xe5;
        }
    }
    Ok(removed)
}

fn read_unit(id: String, image: usize, view: Vec<u8>) -> Result<Unit> {
    let archive = Archive::from_bytes(format!("{id}.fdi"), view.clone())?;
    if archive.inspection().format != "anex86-fdi-hdi" || archive.volumes().len() != 1 {
        return Err("只支持本组已确认的整盘 FDI/FAT12".into());
    }
    let volume = &archive.volumes()[0];
    let fs = volume
        .filesystem
        .as_ref()
        .ok_or_else(|| format!("{id}: 不支持的文件系统：{:?}", volume.diagnostics))?;
    if volume.id != "disk-000-whole" || !fs.fat12.fat_copies_identical {
        return Err("不是整盘 FAT12，或 FAT 副本不一致".into());
    }
    let mut files = BTreeMap::new();
    for file in &fs.files {
        // fivec_new owns geometry/FAT/name parsing. Read its proven physical spans
        // from the same snapshot, without calling its hash-checking read_file API.
        let mut data = Vec::with_capacity(file.size);
        for span in &file.source_ranges {
            if span.file_offset != data.len() {
                return Err("文件来源映射有间隙或重叠".into());
            }
            let end = span
                .source_offset
                .checked_add(span.length)
                .ok_or("来源范围溢出")?;
            data.extend_from_slice(
                view.get(span.source_offset..end)
                    .ok_or("文件来源范围越界")?,
            );
        }
        if data.len() != file.size || files.insert(file.export_path.clone(), data).is_some() {
            return Err("文件长度／导出名称冲突".into());
        }
    }
    Ok(Unit {
        id,
        image,
        parent: None,
        view,
        original_container: None,
        removed_labels: Vec::new(),
        files,
    })
}

pub fn load(game: Game, paths: &[PathBuf], names: Option<&[ImageInfo]>) -> Result<Disks> {
    let expected = if game == Game::Lipstick3 { 5 } else { 2 };
    if paths.len() != expected {
        return Err(format!(
            "{} 需要 {expected} 张盘，选择了 {} 张",
            game.id(),
            paths.len()
        ));
    }
    if names.is_some_and(|n| n.len() != paths.len()) {
        return Err("镜像清单数量不符".into());
    }
    let mut images = Vec::new();
    let mut units = Vec::new();
    for (index, path) in paths.iter().enumerate() {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("FDI 文件名不是 Unicode")?
            .to_owned();
        let info = names.map_or_else(
            || ImageInfo {
                name,
                unit: format!("disk_{}", char::from(b'a' + index as u8)),
            },
            |n| n[index].clone(),
        );
        if info.unit != format!("disk_{}", char::from(b'a' + index as u8))
            || Path::new(&info.name).components().count() != 1
            || info.name.contains(['/', '\\'])
        {
            return Err("非法原盘映射".into());
        }
        let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut view = bytes.clone();
        let removed = label_view(&mut view, game)?;
        let mut unit = read_unit(info.unit.clone(), index, view)?;
        unit.removed_labels = removed;
        units.push(unit);
        images.push(Image { info, bytes });
    }
    if game == Game::Lipstick1 {
        let parent = units
            .iter()
            .find_map(|u| {
                u.files
                    .iter()
                    .find(|(p, _)| p.eq_ignore_ascii_case("XDISKVOL.FDD"))
                    .map(|(p, b)| (u.id.clone(), u.image, p.clone(), b.clone()))
            })
            .ok_or("一代 A 盘缺少 XDISKVOL.FDD")?;
        let mut nested = read_unit(
            "disk_a_xdisk".into(),
            parent.1,
            diskx::logical_fdi(&parent.3)?,
        )?;
        nested.parent = Some((parent.0, parent.2));
        nested.original_container = Some(parent.3);
        units.push(nested);
    }
    let files = all_files(&units);
    let engine = match game {
        Game::Lipstick1 => "ADVG.EXE",
        Game::Lipstick2 => "ADVH.EXE",
        Game::Lipstick3 => "LIP3.OVL",
    };
    if files
        .keys()
        .filter(|p| {
            p.rsplit('/')
                .next()
                .is_some_and(|s| s.eq_ignore_ascii_case(engine))
        })
        .count()
        != 1
    {
        return Err(format!("未找到唯一的本代解释器 {engine}"));
    }
    Ok(Disks { images, units })
}

pub fn all_files(units: &[Unit]) -> BTreeMap<String, Vec<u8>> {
    units
        .iter()
        .flat_map(|u| {
            u.files
                .iter()
                .map(|(p, b)| (format!("{}/{p}", u.id), b.clone()))
        })
        .collect()
}

pub struct Rebuilt {
    pub images: BTreeMap<String, Vec<u8>>,
    pub containers: BTreeMap<String, Vec<u8>>,
}

pub fn rebuild(disks: &Disks, replacements: &BTreeMap<String, Vec<u8>>) -> Result<Rebuilt> {
    let mut changes = BTreeMap::<String, BTreeMap<String, Vec<u8>>>::new();
    for (key, bytes) in replacements {
        let (unit, path) = key.split_once('/').ok_or("资源键缺少盘号")?;
        let source = disks
            .units
            .iter()
            .find(|u| u.id == unit)
            .and_then(|u| u.files.get(path))
            .ok_or("回注资源不属于原盘")?;
        if bytes != source {
            changes
                .entry(unit.into())
                .or_default()
                .insert(path.into(), bytes.clone());
        }
    }
    let mut containers = BTreeMap::new();
    for unit in disks.units.iter().filter(|u| u.parent.is_some()) {
        if let Some(changed) = changes.remove(&unit.id) {
            let logical = pc98_fdi_unpack::rebuild_fdi(&unit.view, "logical.fdi", &changed)?.bytes;
            let packed = diskx::rebuild(
                unit.original_container
                    .as_ref()
                    .ok_or("DiskX 原始输入缺失")?,
                &unit.view,
                &logical,
            )?;
            let (parent, member) = unit.parent.as_ref().ok_or("DiskX 父盘缺失")?;
            containers.insert(format!("{parent}/{member}"), packed.clone());
            changes
                .entry(parent.clone())
                .or_default()
                .insert(member.clone(), packed);
        }
    }
    let mut out = BTreeMap::new();
    for unit in disks.units.iter().filter(|u| u.parent.is_none()) {
        let image = &disks.images[unit.image];
        let bytes = match changes.get(&unit.id) {
            None => image.bytes.clone(),
            Some(changed) => {
                let mut b =
                    pc98_fdi_unpack::rebuild_fdi(&unit.view, &image.info.name, changed)?.bytes;
                for (offset, entry) in &unit.removed_labels {
                    b[*offset..*offset + 32].copy_from_slice(entry);
                }
                b
            }
        };
        out.insert(image.info.name.clone(), bytes);
    }
    Ok(Rebuilt {
        images: out,
        containers,
    })
}
