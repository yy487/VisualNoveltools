#![forbid(unsafe_code)]

mod dialogue;
mod exe;
mod font;
pub mod tdh;

use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use vn_text::Entry;

pub type Result<T> = std::result::Result<T, String>;
const MANIFEST: &str = ".yoshinaga_text_manifest.json";
const TDH: &str = "HTEXT.TDH";
const EXE: &str = "Y2.EXE";

fn io<T>(result: std::io::Result<T>) -> Result<T> {
    result.map_err(|error| error.to_string())
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}
pub fn decode_text(bytes: &[u8], newline: bool) -> Result<String> {
    let text = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(bytes)
        .ok_or("invalid CP932 bytes")?;
    let (encoded, _, failed) = SHIFT_JIS.encode(&text);
    if failed || encoded.as_ref() != bytes {
        return Err("CP932 decoding does not round-trip".into());
    }
    for ch in text.chars() {
        if ch.is_control() && !(newline && ch == '\u{1f}') {
            return Err(format!("unexpected native control U+{:04X}", ch as u32));
        }
    }
    Ok(if newline {
        text.replace('\u{1f}', "\n")
    } else {
        text.into_owned()
    })
}
pub fn encode_text(text: &str, newline: bool) -> Result<Vec<u8>> {
    for (index, ch) in text.chars().enumerate() {
        if ch.is_control() && !(newline && ch == '\n') {
            return Err(format!(
                "character {index}: control U+{:04X} is not allowed",
                ch as u32
            ));
        }
        if SHIFT_JIS.encode(&ch.to_string()).2 {
            return Err(format!(
                "character {index}: {ch:?} cannot be encoded in CP932"
            ));
        }
    }
    let native = if newline {
        text.replace('\n', "\u{1f}")
    } else {
        text.to_string()
    };
    let (bytes, _, failed) = SHIFT_JIS.encode(&native);
    if failed {
        return Err("text cannot be encoded in CP932".into());
    }
    if SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(&bytes)
        .as_deref()
        != Some(native.as_str())
    {
        return Err(
            "text changes during CP932 encoding (use the CP932 equivalent character)".into(),
        );
    }
    Ok(bytes.into_owned())
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UiSlot {
    pub offset: usize,
    pub capacity: usize,
    pub original: String,
    pub category: String,
    pub rule: String,
    pub ea: String,
    pub references: Vec<String>,
}
#[derive(Deserialize, Serialize)]
pub struct UiCatalogue {
    pub source_sha256: String,
    pub entries: Vec<UiSlot>,
}
pub fn ui_catalogue(exe: &[u8]) -> Result<UiCatalogue> {
    let catalogue: UiCatalogue =
        serde_json::from_str(include_str!("ui_catalogue.json")).map_err(|e| e.to_string())?;
    if fivec_new::sha256(exe) != catalogue.source_sha256 {
        return Err("Y2.EXE does not match the disassembled original version".into());
    }
    for slot in &catalogue.entries {
        let bytes = exe
            .get(slot.offset..slot.offset + slot.capacity)
            .ok_or("EXE text slot outside file")?;
        if exe.get(slot.offset + slot.capacity) != Some(&0)
            || decode_text(bytes, false)? != slot.original
        {
            return Err(format!("EXE slot at {} does not match its source", slot.ea));
        }
    }
    Ok(catalogue)
}

pub fn rebuild_ui(exe: &[u8], entries: &[Entry]) -> Result<Vec<u8>> {
    let catalogue = ui_catalogue(exe)?;
    let expected = view(catalogue.entries.iter().map(|slot| slot.original.clone()));
    vn_text::validate_shape(&expected, entries).map_err(|e| e.to_string())?;
    let encoded = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            encode_text(&entry.message, false).map_err(|e| format!("EXE entry {index}: {e}"))
        })
        .collect::<Result<Vec<_>>>()?;
    rebuild_ui_encoded(exe, &encoded)
}

pub fn rebuild_ui_encoded(exe: &[u8], encoded: &[Vec<u8>]) -> Result<Vec<u8>> {
    let catalogue = ui_catalogue(exe)?;
    exe::rebuild(exe, &catalogue.entries, encoded)
}

fn view(messages: impl IntoIterator<Item = String>) -> Vec<Entry> {
    messages
        .into_iter()
        .map(|message| Entry {
            name: None,
            message,
        })
        .collect()
}
fn relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("unsafe relative output path: {}", path.display()));
    }
    Ok(())
}

struct Source {
    path: PathBuf,
    files: BTreeMap<String, Vec<u8>>,
    digest: BTreeMap<String, String>,
    image_sha256: Option<String>,
    image: Option<Vec<u8>>,
}
impl Source {
    fn read(path: &Path) -> Result<Self> {
        let path = io(fs::canonicalize(path))?;
        let mut files = BTreeMap::new();
        let mut image_sha256 = None;
        let mut image = None;
        if path.is_dir() {
            for item in io(fs::read_dir(&path))? {
                let item = io(item)?;
                let metadata = io(fs::symlink_metadata(item.path()))?;
                if !metadata.is_file() || metadata.file_type().is_symlink() {
                    return Err("source must be a flat game directory without links".into());
                }
                let name = item
                    .file_name()
                    .into_string()
                    .map_err(|_| "source filename is not Unicode")?;
                relative(Path::new(&name))?;
                files.insert(name, io(fs::read(item.path()))?);
            }
        } else {
            let bytes = io(fs::read(&path))?;
            let archive = fivec_new::Archive::from_bytes(
                path.file_name()
                    .ok_or("source filename missing")?
                    .to_string_lossy(),
                bytes.clone(),
            )?;
            if archive.volumes().len() != 1 {
                return Err("expected the game's single PC98 partition".into());
            }
            let volume = archive
                .volumes()
                .iter()
                .find(|v| v.id == "disk-000-partition-00")
                .ok_or("expected disk-000-partition-00")?;
            let filesystem = volume
                .filesystem
                .as_ref()
                .ok_or("unsupported image filesystem")?;
            for entry in &filesystem.files {
                relative(Path::new(&entry.export_path))?;
                if Path::new(&entry.export_path).components().count() != 1 {
                    return Err("expected flat game filesystem".into());
                }
                files.insert(
                    entry.export_path.clone(),
                    archive.read_file(&volume.id, &entry.id)?,
                );
            }
            image_sha256 = Some(fivec_new::sha256(&bytes));
            image = Some(bytes);
        }
        ui_catalogue(files.get(EXE).ok_or("source lacks Y2.EXE")?)?;
        if !files.contains_key(TDH) {
            return Err("source lacks HTEXT.TDH".into());
        }
        let digest = files
            .iter()
            .map(|(name, bytes)| (name.clone(), fivec_new::sha256(bytes)))
            .collect();
        Ok(Self {
            path,
            files,
            digest,
            image_sha256,
            image,
        })
    }
    fn verify(&self) -> Result<()> {
        let current = Self::read(&self.path)?;
        if current.digest != self.digest || current.image_sha256 != self.image_sha256 {
            return Err("original source changed after preparation".into());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    schema: String,
    sources: BTreeMap<String, String>,
    image_sha256: Option<String>,
    units: BTreeMap<String, usize>,
}

#[derive(Serialize)]
struct PrimaryText {
    index: usize,
    text_id: usize,
    source_hex: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dialogue_layout: Option<dialogue::Layout>,
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub struct Job {
    source: Source,
    pub output: PathBuf,
    pub protected_inputs: Vec<PathBuf>,
    pub details: Vec<String>,
    files: BTreeMap<String, Vec<u8>>,
    translations: Vec<(PathBuf, Vec<u8>)>,
}

fn output_path(inputs: &[PathBuf], output: &Path) -> Result<PathBuf> {
    let absolute = io(std::path::absolute(output))?;
    let parent = io(fs::canonicalize(
        absolute.parent().ok_or("output has no parent")?,
    ))?;
    let output = parent.join(absolute.file_name().ok_or("output has no name")?);
    if io(output.try_exists())? {
        return Err("output already exists; choose a new directory".into());
    }
    fn key(path: &Path) -> String {
        path.to_string_lossy().replace('\\', "/").to_lowercase()
    }
    let destination = key(&output);
    for input in inputs {
        let input = key(&io(fs::canonicalize(input))?);
        if destination == input
            || destination.starts_with(&(input.clone() + "/"))
            || input.starts_with(&(destination.clone() + "/"))
        {
            return Err("output overlaps a protected input".into());
        }
    }
    Ok(output)
}
impl Job {
    pub fn export(source: &Path, output: &Path) -> Result<Self> {
        let source = Source::read(source)?;
        let archive = tdh::Tdh::parse(&source.files[TDH])?;
        let catalogue = ui_catalogue(&source.files[EXE])?;
        let messages = archive.messages()?;
        let projected = messages
            .iter()
            .map(|message| dialogue::project(message))
            .collect::<Vec<_>>();
        let text_view = projected
            .iter()
            .map(|(entry, _)| entry.clone())
            .collect::<Vec<_>>();
        let ui_view = view(catalogue.entries.iter().map(|slot| slot.original.clone()));
        let primary = archive
            .texts
            .iter()
            .zip(&messages)
            .enumerate()
            .map(|(index, (bytes, message))| PrimaryText {
                index,
                text_id: index + 1,
                source_hex: hex(bytes),
                message: message.clone(),
                dialogue_layout: projected[index].1.clone(),
            })
            .collect::<Vec<_>>();
        let manifest = Manifest {
            schema: "yoshinaga-text-v2".into(),
            sources: source.digest.clone(),
            image_sha256: source.image_sha256.clone(),
            units: BTreeMap::from([(TDH.into(), text_view.len()), (EXE.into(), ui_view.len())]),
        };
        let files = BTreeMap::from([
            ("primary/HTEXT.TDH.json".into(), json(&primary)?),
            ("primary/Y2.EXE.json".into(), json(&catalogue)?),
            (
                "translations/HTEXT.TDH.json".into(),
                vn_text::write_template(&text_view).map_err(|e| e.to_string())?,
            ),
            (
                "translations/Y2.EXE.json".into(),
                vn_text::write_template(&ui_view).map_err(|e| e.to_string())?,
            ),
            (MANIFEST.into(), json(&manifest)?),
        ]);
        let protected_inputs = vec![source.path.clone()];
        let output = output_path(&protected_inputs, output)?;
        let details = vec![
            format!(
                "文本库: {} 条，EXE 硬编码文字: {} 条",
                text_view.len(),
                ui_view.len()
            ),
            "明确的说话者拆为可编辑 name/message；原冒号与排版缩进由一次记录还原。EXE 文字仍用固定槽。"
                .into(),
        ];
        Ok(Self {
            source,
            output,
            protected_inputs,
            details,
            files,
            translations: Vec::new(),
        })
    }

    pub fn import(source: &Path, workspace: &Path, output: &Path) -> Result<Self> {
        Self::import_with_font(source, workspace, output, None, vn_font::font_98::FONT_FACE)
    }

    pub fn import_with_font(
        source: &Path,
        workspace: &Path,
        output: &Path,
        font_source: Option<&Path>,
        face: &str,
    ) -> Result<Self> {
        Self::import_with_options(source, workspace, output, font_source, face, None)
    }

    pub fn import_with_options(
        source: &Path,
        workspace: &Path,
        output: &Path,
        font_source: Option<&Path>,
        face: &str,
        base_hdi: Option<&Path>,
    ) -> Result<Self> {
        let source = Source::read(source)?;
        let workspace = io(fs::canonicalize(workspace))?;
        let manifest_path = workspace.join(MANIFEST);
        let manifest_bytes = io(fs::read(&manifest_path))?;
        let manifest: Manifest =
            serde_json::from_slice(&manifest_bytes).map_err(|e| e.to_string())?;
        if !matches!(
            manifest.schema.as_str(),
            "yoshinaga-text-v1" | "yoshinaga-text-v2"
        ) || manifest.sources != source.digest
            || manifest.image_sha256 != source.image_sha256
        {
            return Err("translation workspace is bound to a different original source".into());
        }
        let archive = tdh::Tdh::parse(&source.files[TDH])?;
        let catalogue = ui_catalogue(&source.files[EXE])?;
        let messages = archive.messages()?;
        let projected = messages
            .iter()
            .map(|message| {
                if manifest.schema == "yoshinaga-text-v2" {
                    dialogue::project(message)
                } else {
                    (
                        Entry {
                            name: None,
                            message: message.clone(),
                        },
                        None,
                    )
                }
            })
            .collect::<Vec<_>>();
        let expected_text = projected
            .iter()
            .map(|(entry, _)| entry.clone())
            .collect::<Vec<_>>();
        let expected_ui = view(catalogue.entries.iter().map(|slot| slot.original.clone()));
        let units = BTreeMap::from([
            (TDH.to_string(), expected_text.len()),
            (EXE.to_string(), expected_ui.len()),
        ]);
        if manifest.units != units {
            return Err("translation unit mapping changed".into());
        }
        let mut translations = vec![(manifest_path, manifest_bytes)];
        let mut game_files = source.files.clone();
        let mut final_text = expected_text.clone();
        let mut final_ui = expected_ui.clone();
        let mut selected = 0;
        let mut edited = 0;
        for (name, expected) in [(TDH, &expected_text), (EXE, &expected_ui)] {
            let path = workspace.join("translations").join(format!("{name}.json"));
            if !io(path.try_exists())? {
                continue;
            }
            let bytes = io(fs::read(&path))?;
            let supplied = vn_text::read_template(&bytes).map_err(|e| format!("{name}: {e}"))?;
            vn_text::validate_shape(expected, &supplied).map_err(|e| format!("{name}: {e}"))?;
            edited += expected
                .iter()
                .zip(&supplied)
                .filter(|(a, b)| a != b)
                .count();
            if name == TDH {
                final_text = supplied;
            } else {
                final_ui = supplied;
            }
            translations.push((path, bytes));
            selected += 1;
        }
        if selected == 0 {
            return Err("no translation JSON selected".into());
        }
        let mut protected_inputs = vec![source.path.clone(), workspace];
        let original_image = if let Some(path) = base_hdi {
            let baseline = Source::read(path)?;
            if baseline.digest != source.digest
                || baseline.image.is_none()
                || source
                    .image_sha256
                    .as_ref()
                    .is_some_and(|hash| Some(hash) != baseline.image_sha256.as_ref())
            {
                return Err("重封基准 HDI 与原始游戏来源不一致".into());
            }
            let bytes = baseline.image.ok_or("重封基准必须是 HDI 文件")?;
            protected_inputs.push(baseline.path.clone());
            translations.push((baseline.path, bytes.clone()));
            Some(bytes)
        } else {
            source.image.clone()
        };
        let font_bytes = if let Some(path) = font_source {
            let path = io(fs::canonicalize(path))?;
            let bytes = io(fs::read(&path))?;
            protected_inputs.push(path.clone());
            translations.push((path, bytes.clone()));
            bytes
        } else {
            source
                .files
                .iter()
                .find(|(name, _)| {
                    name.eq_ignore_ascii_case("font.bmp") || name.eq_ignore_ascii_case("font.tmp")
                })
                .map(|(_, bytes)| bytes.clone())
                .unwrap_or_else(|| vn_font::font_98::EMBEDDED_FONT.to_vec())
        };
        let native_text = final_text
            .iter()
            .zip(&projected)
            .enumerate()
            .map(|(index, (entry, (_, layout)))| {
                dialogue::merge(entry, layout.as_ref())
                    .map_err(|e| format!("TDH entry {index}: {e}"))
            })
            .collect::<Result<Vec<_>>>()?;
        let font = font::prepare(&view(native_text.clone()), &final_ui, &font_bytes, face)?;
        let encoded_text = final_text
            .iter()
            .zip(&expected_text)
            .zip(&archive.texts)
            .enumerate()
            .map(|(index, ((new, old), raw))| {
                if new == old {
                    Ok(raw.clone())
                } else {
                    font::encode(&native_text[index], true, font.plan.as_ref())
                        .map_err(|e| format!("TDH entry {index}: {e}"))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        let encoded_ui = final_ui
            .iter()
            .zip(&expected_ui)
            .zip(&catalogue.entries)
            .enumerate()
            .map(|(index, ((new, old), slot))| {
                if new == old {
                    Ok(source.files[EXE][slot.offset..slot.offset + slot.capacity].to_vec())
                } else {
                    font::encode(&new.message, false, font.plan.as_ref())
                        .map_err(|e| format!("EXE entry {index}: {e}"))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        game_files.insert(TDH.into(), archive.rebuild_encoded(&encoded_text)?);
        game_files.insert(
            EXE.into(),
            rebuild_ui_encoded(&source.files[EXE], &encoded_ui)?,
        );
        let hashes: BTreeMap<_, _> = game_files
            .iter()
            .map(|(name, bytes)| (name.clone(), fivec_new::sha256(bytes)))
            .collect();
        let rebuilt_hdi = if let Some(image) = original_image {
            let rebuilt = platinum_star_hdi_tool::rebuild_hdi_bytes(&image, &game_files)?;
            // Independent supported reader verifies every repacked file, not only modified ones.
            let archive = fivec_new::Archive::from_bytes("game.hdi", rebuilt.clone())?;
            let volume = archive
                .volumes()
                .iter()
                .find(|volume| volume.id == "disk-000-partition-00")
                .ok_or("重封后卷 ID 不匹配")?;
            let filesystem = volume.filesystem.as_ref().ok_or("重封后文件系统不受支持")?;
            if filesystem.files.len() != game_files.len() {
                return Err("重封后文件数不一致".into());
            }
            for file in &filesystem.files {
                if game_files.get(&file.export_path).map(Vec::as_slice)
                    != Some(archive.read_file(&volume.id, &file.id)?.as_slice())
                {
                    return Err(format!("重封后文件核对失败: {}", file.export_path));
                }
            }
            Some(rebuilt)
        } else {
            None
        };
        let packed_hash = rebuilt_hdi.as_ref().map(|bytes| fivec_new::sha256(bytes));
        let report = serde_json::json!({"schema":"yoshinaga-import-v1", "modified_entries":edited, "sources":source.digest,"image_sha256":source.image_sha256, "game_files":hashes,"rebuilt_hdi_sha256":packed_hash,"font98_patched_glyphs":font.patched, "limits":{"tdh_entry_bytes":2047,"tdh_compressed_read_bytes":1024,"exe_text":"confirmed string slots and verified padding relocation; disk-prompt code adjusted to actual encoded lengths"}});
        let mut files = game_files
            .into_iter()
            .map(|(name, bytes)| (format!("game/{name}"), bytes))
            .collect::<BTreeMap<_, _>>();
        files.insert("import_report.json".into(), json(&report)?);
        let has_hdi = rebuilt_hdi.is_some();
        if let Some(image) = rebuilt_hdi {
            files.insert("game.hdi".into(), image);
        }
        files.extend(font.files);
        let output = output_path(&protected_inputs, output)?;
        let details = vec![
            format!("选择 {selected} 份 JSON，修改 {edited} 条文字；输出完整游戏目录。"),
            "文本编号、脚本指令保持；TDH 更新字符表/Huffman 树及全部 bit 偏移。".into(),
            format!(
                "font98: {} 个补丁字形；有补丁时输出独立 NP2 字库与映射记录。",
                font.patched
            ),
            if has_hdi {
                "已重封 game.hdi，并用 fivec-new 独立读取核对全部文件。".into()
            } else {
                "原始来源为目录且未指定 base-hdi，只生成游戏目录；可补选原盘进行重封。".into()
            },
        ];
        Ok(Self {
            source,
            output,
            protected_inputs,
            details,
            files,
            translations,
        })
    }

    fn verify_inputs(&self) -> Result<()> {
        self.source.verify()?;
        for (path, bytes) in &self.translations {
            if io(fs::read(path))? != *bytes {
                return Err("translation input changed after preparation".into());
            }
        }
        if io(self.output.try_exists())? {
            return Err("output appeared after preparation".into());
        }
        Ok(())
    }

    pub fn execute(self) -> Result<usize> {
        self.verify_inputs()?;
        let parent = self.output.parent().ok_or("output parent missing")?;
        let stage = Stage::create(parent)?;
        for (name, bytes) in &self.files {
            relative(Path::new(name))?;
            let path = stage.0.join(name);
            io(fs::create_dir_all(
                path.parent().ok_or("staged parent missing")?,
            ))?;
            let mut file = io(OpenOptions::new().write(true).create_new(true).open(&path))?;
            io(file.write_all(bytes))?;
            io(file.sync_all())?;
            drop(file);
            if io(fs::read(&path))? != *bytes {
                return Err(format!("staged verification failed: {name}"));
            }
        }
        self.verify_inputs()?;
        io(fs::rename(&stage.0, &self.output))?;
        Ok(self.files.len())
    }
}

struct Stage(PathBuf);
impl Stage {
    fn create(parent: &Path) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".yoshinaga-stage-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.to_string()),
            }
        }
        Err("cannot allocate staging directory".into())
    }
}
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn newline_encoding_and_invalid_controls() {
        assert_eq!(
            encode_text("A\n校", true).unwrap(),
            [0x41, 0x1f, 0x8d, 0x5a]
        );
        assert_eq!(
            decode_text(&[0x41, 0x1f, 0x8d, 0x5a], true).unwrap(),
            "A\n校"
        );
        for bad in ["\0", "\r", "\t", "😀"] {
            assert!(encode_text(bad, true).is_err());
        }
    }
    #[test]
    fn library_output_protects_inputs_and_existing_outputs() {
        let root = std::env::temp_dir().join(format!("yoshinaga-io-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let input = root.join("input");
        fs::create_dir(&input).unwrap();
        assert!(output_path(std::slice::from_ref(&input), &input.join("inside")).is_err());
        assert!(output_path(std::slice::from_ref(&input), &input).is_err());
        assert!(output_path(std::slice::from_ref(&input), &root.join("new")).is_ok());
        let stage = Stage::create(&root).unwrap();
        let stage_path = stage.0.clone();
        drop(stage);
        assert!(!stage_path.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
