#![forbid(unsafe_code)]

pub mod archive;
pub mod disk;
mod output;
pub mod text;
pub mod workflow;

pub use output::Prepared;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, String>;
pub const EXE_SHA256: &str = "084005a5acd29e10f2ffd880617dd8f01bfa7285c49f56ef0ea6f6a2d5fb88af";

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Clone, Copy)]
pub enum Mode {
    Unpack,
    Extract,
}

#[derive(Serialize)]
struct Manifest<'a> {
    schema: &'static str,
    tool_version: &'static str,
    mode: &'static str,
    executable_sha256: &'static str,
    references: &'a [archive::Reference],
    resources: &'a [archive::Resource],
    translation_files: usize,
    translation_entries: usize,
    limitations: Vec<&'static str>,
}

#[derive(Clone)]
pub(crate) struct Source {
    pub path: PathBuf,
    pub hash: String,
}

pub(crate) struct Game {
    pub sources: Vec<Source>,
    pub originals: BTreeMap<String, Vec<u8>>,
    pub images: Vec<Vec<u8>>,
    pub inspection: BTreeMap<String, Vec<u8>>,
}

pub(crate) fn load(disk1: &Path, disk2: &Path) -> Result<Game> {
    let mut game = Game {
        sources: Vec::new(),
        originals: BTreeMap::new(),
        images: Vec::new(),
        inspection: BTreeMap::new(),
    };
    for (label, path) in [("disk1", disk1), ("disk2", disk2)] {
        let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        let hash = fivec_new::sha256(&bytes);
        let image = fivec_new::Archive::from_bytes(
            path.file_name()
                .ok_or("image has no name")?
                .to_string_lossy(),
            bytes.clone(),
        )?;
        if image.volumes().len() != 1 || image.volumes()[0].id != "disk-000-whole" {
            return Err("expected whole-disk FAT12".into());
        }
        let v = &image.volumes()[0];
        for f in &v.filesystem.as_ref().ok_or("unsupported filesystem")?.files {
            game.originals.insert(
                format!("{label}/{}", f.export_path),
                image.read_file(&v.id, &f.id)?,
            );
        }
        game.inspection
            .insert(format!("sources/{label}.json"), json(image.inspection())?);
        game.sources.push(Source { path, hash });
        game.images.push(bytes);
    }
    if game.sources[0].path == game.sources[1].path {
        return Err("disk1 and disk2 must be distinct".into());
    }
    let exe = game
        .originals
        .get("disk1/KOHAKU.COM")
        .ok_or("missing KOHAKU.COM; check disk order")?;
    if fivec_new::sha256(exe) != EXE_SHA256 {
        return Err("unrecognized KOHAKU.COM version".into());
    }
    if !game.originals.contains_key("disk1/DISK1.DAT")
        || !game.originals.contains_key("disk2/DISK2.DAT")
    {
        return Err("missing DAT; check disk order".into());
    }
    Ok(game)
}

/// Parse immutable image snapshots through the shared filesystem library.
/// No external extractor, executable or scan-based string guessing is used.
pub fn prepare(disk1: &Path, disk2: &Path, output: &Path, mode: Mode) -> Result<Prepared> {
    let game = load(disk1, disk2)?;
    let originals = &game.originals;
    let mut files = game.inspection.clone();
    for (path, bytes) in originals {
        files.insert(format!("files/{path}"), bytes.clone());
    }
    let get = |name: &str| {
        originals
            .get(name)
            .ok_or_else(|| format!("missing {name}; check disk order"))
    };
    let exe = get("disk1/KOHAKU.COM")?;
    if fivec_new::sha256(exe) != EXE_SHA256 {
        return Err(
            "unrecognized KOHAKU.COM version; resource index locations require analysis".into(),
        );
    }
    let catalog = archive::parse(exe, get("disk1/DISK1.DAT")?, get("disk2/DISK2.DAT")?)?;
    let mut translation_files = 0;
    let mut translation_entries = 0;
    for resource in &catalog.resources {
        files.insert(resource.path.clone(), resource.bytes.clone());
        if matches!(mode, Mode::Extract) && resource.kind.ends_with("-text") {
            let translation = text::parse(
                &resource.path,
                &resource.archive,
                resource.offset,
                &resource.bytes,
                &resource.kind,
                true,
            )?;
            translation_entries += translation.entries.len();
            translation_files += 1;
            files.insert(
                resource
                    .path
                    .replace("resources/", "json/")
                    .replace(".bin", ".json"),
                json(&translation)?,
            );
        }
    }
    if matches!(mode, Mode::Extract) {
        // Referenced by CS:3E39 (menus) and CS:3B6E (labels).
        // Exact range ends before the independent bitmap data at CS:92DD.
        for (name, begin, end, kind) in [
            ("menu", 0x844b, 0x888f, "menu"),
            ("labels", 0x888f, 0x91dd, "ui-label"),
        ] {
            let file = format!("KOHAKU.COM/{name}");
            let translation =
                text::parse(&file, "KOHAKU.COM", begin, &exe[begin..end], kind, false)?;
            translation_entries += translation.entries.len();
            translation_files += 1;
            files.insert(format!("json/KOHAKU.COM/{name}.json"), json(&translation)?);
        }
        files.insert(
            "font/reference_font.tmp".into(),
            vn_font::font_98::EMBEDDED_FONT.to_vec(),
        );
    }
    let manifest = Manifest {
        schema: "kohaku-unpack-v1",
        tool_version: env!("CARGO_PKG_VERSION"),
        mode: if matches!(mode, Mode::Extract) { "extract" } else { "unpack" },
        executable_sha256: EXE_SHA256,
        references: &catalog.references,
        resources: &catalog.resources,
        translation_files,
        translation_entries,
        limitations: vec![
            "Import supports bounded text rebuilding, PC98 font and FDI reconstruction; no full-game emulator playthrough.",
            "Image lettering and audio remain unchanged.",
            "Executable text scope is the two verified JIS tables; DOS errors and other embedded text are not exported.",
            "Fourteen music identifiers are preserved in slots but excluded from editable entries.",
            "Only the documented KOHAKU.COM hash and complete indexed DAT layouts are accepted.",
        ],
    };
    files.insert("manifest.json".into(), json(&manifest)?);
    Prepared::new(
        game.sources,
        output,
        files,
        catalog.resources.len(),
        translation_files,
        translation_entries,
    )
}

fn json(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests;
