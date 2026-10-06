//! Translation merge, shared NP2 font planning, DAT/COM and FDI rebuild.
use crate::{archive, disk, json, load, text, Prepared, Result, Source};
use encoding_rs::SHIFT_JIS;
use serde_json::{json as value, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use vn_font::font_98::{self, EncodingPlan, SubstitutionMap};

pub type RebuiltArchives = (Vec<u8>, BTreeMap<String, Vec<u8>>);

pub(crate) fn walk(path: &Path) -> Result<Vec<PathBuf>> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err(format!("linked input is unsupported: {}", path.display()));
        }
    }
    if meta.file_type().is_symlink() {
        return Err("linked input is unsupported".into());
    }
    if meta.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    let mut out = Vec::new();
    for child in fs::read_dir(path).map_err(|e| e.to_string())? {
        out.extend(walk(&child.map_err(|e| e.to_string())?.path())?);
    }
    out.sort();
    Ok(out)
}

pub(crate) fn source_hash(path: &Path) -> Result<String> {
    if path.is_file() {
        return Ok(fivec_new::sha256(
            &fs::read(path).map_err(|e| e.to_string())?,
        ));
    }
    let mut inventory = Vec::new();
    for p in walk(path)? {
        inventory.push((
            p.strip_prefix(path)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned(),
            fivec_new::sha256(&fs::read(&p).map_err(|e| e.to_string())?),
        ));
    }
    Ok(fivec_new::sha256(&json(&inventory)?))
}

fn documents(
    catalog: &archive::Catalog,
    exe: &[u8],
) -> Result<BTreeMap<String, text::Translation>> {
    let mut docs = BTreeMap::new();
    for r in &catalog.resources {
        if r.kind.ends_with("-text") {
            docs.insert(
                r.path.clone(),
                text::parse(&r.path, &r.archive, r.offset, &r.bytes, &r.kind, true)?,
            );
        }
    }
    for (name, start, end, kind) in [
        ("menu", 0x844b, 0x888f, "menu"),
        ("labels", 0x888f, 0x91dd, "ui-label"),
    ] {
        let key = format!("KOHAKU.COM/{name}");
        docs.insert(
            key.clone(),
            text::parse(&key, "KOHAKU.COM", start, &exe[start..end], kind, false)?,
        );
    }
    Ok(docs)
}

pub fn merge_document(
    docs: &mut BTreeMap<String, text::Translation>,
    supplied: &Value,
    selected: &mut BTreeSet<(String, usize)>,
) -> Result<()> {
    let object = supplied
        .as_object()
        .ok_or("translation must be an object")?;
    let keys: Vec<String> = docs
        .iter()
        .filter(|(_, d)| {
            supplied.get("source").and_then(Value::as_str) == Some(d.source.as_str())
                && supplied.get("source_offset").and_then(Value::as_u64)
                    == Some(d.source_offset as u64)
                && supplied.get("source_sha256").and_then(Value::as_str)
                    == Some(d.source_sha256.as_str())
        })
        .map(|(k, _)| k.clone())
        .collect();
    if keys.len() != 1 {
        return Err("translation source identity/hash does not match original baseline".into());
    }
    let key = &keys[0];
    let doc = docs.get_mut(key).ok_or("missing baseline")?;
    let baseline = serde_json::to_value(&*doc).map_err(|e| e.to_string())?;
    for (field, value) in object {
        if field != "entries" && baseline.get(field) != Some(value) {
            return Err(format!("{key}: immutable/unknown field {field}"));
        }
    }
    let entries = object
        .get("entries")
        .and_then(Value::as_array)
        .ok_or("entries must be an array")?;
    for supplied in entries {
        let entry = supplied.as_object().ok_or("entry must be an object")?;
        let original = entry
            .get("scr_msg")
            .and_then(Value::as_str)
            .ok_or("scr_msg is required")?;
        let message = entry
            .get("message")
            .and_then(Value::as_str)
            .ok_or("message must be a string")?;
        validate_message(message)?;
        let matching: Vec<usize> = doc
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                if let Some(index) = entry.get("_index") {
                    index.as_u64() == Some(e._index as u64)
                } else if let Some(offset) = entry.get("_offset") {
                    offset.as_u64() == Some(e._offset as u64)
                } else {
                    e.scr_msg == original
                }
            })
            .map(|(i, _)| i)
            .collect();
        if matching.len() != 1 {
            return Err(format!("{key}: missing/ambiguous entry locator"));
        }
        let target = &mut doc.entries[matching[0]];
        if target.scr_msg != original {
            return Err(format!(
                "{key} #{}: scr_msg differs from original",
                target._index
            ));
        }
        let expected = serde_json::to_value(&*target).map_err(|e| e.to_string())?;
        for (field, value) in entry {
            if field != "message" && expected.get(field) != Some(value) {
                return Err(format!(
                    "{key} #{}: immutable/unknown field {field}",
                    target._index
                ));
            }
        }
        if !selected.insert((key.clone(), target._index)) {
            return Err(format!("{key} #{} supplied more than once", target._index));
        }
        if doc.source == "KOHAKU.COM" && message.contains('\n') {
            return Err("COM menu/label text cannot contain line breaks".into());
        }
        target.message = message.into();
    }
    Ok(())
}

pub fn validate_message(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err("empty translations would turn text into structural empty slots".into());
    }
    if let Some(c) = s
        .chars()
        .find(|c| (*c != '\n' && c.is_control()) || *c as u32 > 0xffff)
    {
        return Err(format!(
            "unsupported character U+{:04X}; only LF is an allowed control",
            c as u32
        ));
    }
    Ok(())
}

/// Main dialogue window starts at y=290, uses 31 columns and 20px line pitch.
pub fn validate_dialogue_layout(s: &str) -> Result<()> {
    let (mut row, mut column) = (0, 0);
    for c in s.chars() {
        if column == 31 {
            row += 1;
            column = 0;
        }
        if c == '\n' {
            row += 1;
            column = 0;
        } else {
            if row >= 5 {
                return Err(
                    "dialogue exceeds five 31-character lines; shorten text or adjust line breaks"
                        .into(),
                );
            }
            column += 1;
        }
    }
    Ok(())
}

fn reserve_text(s: &str, reserved: &mut BTreeSet<u16>) {
    for c in s.chars() {
        if let Ok(cp) = font_98::cp932_for_carrier(c) {
            if font_98::has_loaded_np2_slot(c) {
                reserved.insert(u16::from_be_bytes(cp));
            }
        }
    }
}

pub fn encode_message(message: &str, plan: &EncodingPlan) -> Result<Vec<u8>> {
    validate_message(message)?;
    let mut encoded = Vec::new();
    for c in message.chars() {
        if c == '\n' {
            encoded.extend([0x21, 0x77]);
        } else {
            encoded.extend(font_98::cp932_to_jis(font_98::cp932_for_carrier(
                plan.carrier_for(c)?,
            )?)?);
        }
    }
    Ok(encoded)
}

pub fn rebuild_text(doc: &text::Translation, plan: &EncodingPlan) -> Result<Vec<u8>> {
    let entries: BTreeMap<usize, &text::TextEntry> =
        doc.entries.iter().map(|e| (e._index, e)).collect();
    let mut out = Vec::new();
    for slot in &doc.slots {
        let body = if let Some(entry) = entries.get(&slot.index) {
            if entry.message != entry.scr_msg {
                if doc.source == "DISK1.DAT" && entry._kind != "profile-text" {
                    validate_dialogue_layout(&entry.message)
                        .map_err(|e| format!("{} #{}: {e}", entry._file, entry._index))?;
                }
                encode_message(&entry.message, plan)?
            } else {
                unhex(&slot.raw_hex)?
            }
        } else {
            unhex(&slot.raw_hex)?
        };
        out.extend(body);
        out.extend([0, 0]);
    }
    Ok(out)
}

fn unhex(s: &str) -> Result<Vec<u8>> {
    if !s.is_ascii() || !s.len().is_multiple_of(2) {
        return Err("invalid hex".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

fn capacity(resource: &archive::Resource) -> usize {
    if resource.path.ends_with("scene_000.bin") {
        0x1b00
    } else if resource.kind == "scene-text" {
        0x500
    } else if resource.kind == "profile-text" || resource.kind == "common-text" {
        0x5500
    } else {
        0x5000
    }
}

/// DAT references are all rewritten, including physical aliases and later resources.
pub fn rebuild_archives(
    catalog: &archive::Catalog,
    exe: &[u8],
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<RebuiltArchives> {
    let mut patched = exe.to_vec();
    let mut archives = BTreeMap::new();
    for name in ["DISK1.DAT", "DISK2.DAT"] {
        let mut bytes = Vec::new();
        for r in catalog.resources.iter().filter(|r| r.archive == name) {
            let body = replacements.get(&r.path).unwrap_or(&r.bytes);
            if body.is_empty()
                || body.len() > u16::MAX as usize
                || (r.kind.ends_with("-text") && body.len() > capacity(r))
            {
                return Err(format!(
                    "{}: {} bytes exceeds verified load buffer {}",
                    r.path,
                    body.len(),
                    capacity(r)
                ));
            }
            let offset = bytes.len();
            bytes.extend(body);
            for &idx in &r.references {
                let at = catalog.references[idx].executable_file_offset;
                for (j, v) in [
                    ((offset >> 16) as u16),
                    (offset as u16),
                    (body.len() as u16),
                ]
                .into_iter()
                .enumerate()
                {
                    patched[at + j * 2..at + j * 2 + 2].copy_from_slice(&v.to_le_bytes());
                }
            }
        }
        archives.insert(name.into(), bytes);
    }
    let verified = archive::parse(&patched, &archives["DISK1.DAT"], &archives["DISK2.DAT"])?;
    if verified.resources.len() != catalog.resources.len() {
        return Err("DAT physical resource set changed".into());
    }
    for (old, new) in catalog.resources.iter().zip(&verified.resources) {
        if old.path != new.path
            || new.bytes != *replacements.get(&old.path).unwrap_or(&old.bytes)
            || old.references != new.references
        {
            return Err("DAT re-extraction mismatch".into());
        }
    }
    Ok((patched, archives))
}

pub struct ImportOptions<'a> {
    pub disk1: &'a Path,
    pub disk2: &'a Path,
    pub translations: &'a [PathBuf],
    pub font: Option<&'a Path>,
    pub face: &'a str,
    pub output: &'a Path,
}

pub fn prepare_import(options: ImportOptions<'_>) -> Result<Prepared> {
    let mut game = load(options.disk1, options.disk2)?;
    let exe = &game.originals["disk1/KOHAKU.COM"];
    let catalog = archive::parse(
        exe,
        &game.originals["disk1/DISK1.DAT"],
        &game.originals["disk2/DISK2.DAT"],
    )?;
    let mut docs = documents(&catalog, exe)?;
    let mut selected = BTreeSet::new();
    let mut picked = BTreeSet::new();
    for p in options.translations {
        let path = fs::canonicalize(p).map_err(|e| e.to_string())?;
        let hash = source_hash(&path)?;
        for file in walk(&path)? {
            if file
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("json"))
            {
                if !picked.insert(file.clone()) {
                    return Err("overlapping translation selections".into());
                }
                let data = fs::read(&file).map_err(|e| e.to_string())?;
                let supplied: Value = serde_json::from_slice(&data)
                    .map_err(|e| format!("{}: {e}", file.display()))?;
                merge_document(&mut docs, &supplied, &mut selected)
                    .map_err(|e| format!("{}: {e}", file.display()))?;
            }
        }
        game.sources.push(Source { path, hash });
    }
    if picked.is_empty() {
        return Err("select at least one translation JSON file/directory".into());
    }
    let font = if let Some(path) = options.font {
        let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        game.sources.push(Source {
            path,
            hash: fivec_new::sha256(&bytes),
        });
        bytes
    } else {
        font_98::EMBEDDED_FONT.to_vec()
    };
    font_98::validate_font(&font)?;
    let mut reserved = BTreeSet::new();
    let mut final_texts = Vec::new();
    let mut plan_texts = Vec::new();
    for doc in docs.values() {
        for e in &doc.entries {
            if e.message == e.scr_msg {
                reserve_text(&e.scr_msg, &mut reserved);
            } else {
                plan_texts.push(e.message.replace('\n', ""));
            }
            final_texts.push(e.message.replace('\n', ""));
        }
        for slot in doc.slots.iter().filter(|s| s.kind == "resource-label") {
            reserve_text(&text::decode(&unhex(&slot.raw_hex)?)?, &mut reserved);
        }
    }
    // Additional conservative protection for COM strings outside verified tables.
    // These bytes are never exported as guessed text; over-reservation is intentional.
    for (i, pair) in exe.windows(2).enumerate() {
        if (0x844b..0x91dd).contains(&i) {
            continue;
        }
        for cp in [
            Some([pair[0], pair[1]]),
            font_98::jis_to_cp932([pair[0], pair[1]]).ok(),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(s) = SHIFT_JIS.decode_without_bom_handling_and_without_replacement(&cp) {
                reserve_text(&s, &mut reserved);
            }
        }
    }
    let mut forbidden = vec![u16::from_be_bytes(font_98::jis_to_cp932([0x21, 0x77])?)];
    for cell in 0x41..=0x46 {
        forbidden.push(u16::from_be_bytes(font_98::jis_to_cp932([0x23, cell])?));
    }
    let plan = EncodingPlan::build_with_forbidden_cp932(
        &SubstitutionMap::embedded()?,
        reserved.iter().copied(),
        forbidden,
        plan_texts.iter().map(String::as_str),
    )?;
    let font_build = font_98::prepare_font(&font, &plan.requests(), &reserved, options.face)?;
    let mut replacements = BTreeMap::new();
    let mut patched_exe = exe.clone();
    let mut report_entries = Vec::new();
    let mut files = BTreeMap::new();
    for (key, doc) in &docs {
        if doc.entries.iter().all(|e| e.message == e.scr_msg) {
            continue;
        }
        let bytes = rebuild_text(doc, &plan)?;
        let parsed = text::parse(
            key,
            &doc.source,
            doc.source_offset,
            &bytes,
            "text",
            doc.source != "KOHAKU.COM",
        )?;
        if parsed.slots.len() != doc.slots.len() || parsed.entries.len() != doc.entries.len() {
            return Err(format!("{key}: structural text slots changed"));
        }
        for (expected, actual) in doc.entries.iter().zip(&parsed.entries) {
            let display = plan.decode_carriers(&actual.scr_msg);
            if display != expected.message {
                return Err(format!(
                    "{key} #{}: display re-extraction mismatch",
                    expected._index
                ));
            }
            if expected.message != expected.scr_msg {
                report_entries.push(value!({"file":key,"index":expected._index,"original":expected.scr_msg,"message":expected.message,"reextracted":display}));
            }
        }
        if doc.source == "KOHAKU.COM" {
            let old_len = doc.slots.last().ok_or("no slots")?.offset
                + doc.slots.last().ok_or("no slots")?.byte_length
                + 2;
            if bytes.len() > old_len {
                return Err(format!(
                    "{key}: fixed COM table capacity {old_len} bytes; translation requires {}",
                    bytes.len()
                ));
            }
            let target = &mut patched_exe[doc.source_offset..doc.source_offset + old_len];
            target.fill(0);
            target[..bytes.len()].copy_from_slice(&bytes);
        } else {
            replacements.insert(key.clone(), bytes);
        }
        files.insert(format!("review/{}.json", key.replace('/', "_")), json(doc)?);
    }
    let (patched_exe, archives) = rebuild_archives(&catalog, &patched_exe, &replacements)?;
    let mut d1 = BTreeMap::new();
    d1.insert("KOHAKU.COM".into(), patched_exe.clone());
    d1.insert("DISK1.DAT".into(), archives["DISK1.DAT"].clone());
    let d2 = BTreeMap::from([("DISK2.DAT".into(), archives["DISK2.DAT"].clone())]);
    let images = [
        disk::rebuild(&game.images[0], &d1)?,
        disk::rebuild(&game.images[1], &d2)?,
    ];
    for (path, bytes) in &game.originals {
        let (disk, name) = path.split_once('/').ok_or("original path")?;
        let replacement = if disk == "disk1" {
            d1.get(name)
        } else {
            d2.get(name)
        };
        files.insert(
            format!("files/{path}"),
            replacement.unwrap_or(bytes).clone(),
        );
    }
    for (i, bytes) in images.iter().enumerate() {
        files.insert(format!("images/disk{}.fdi", i + 1), bytes.clone());
    }
    files.insert("font.tmp".into(), font_build.bytes);
    files.insert("font_mapping.json".into(), json(&plan.manifest_entries()?)?);
    files.insert("font_request.json".into(),json(&value!({"schema_version":1,"backend":"pc98-np2","final_texts":final_texts,"reserved_cp932":reserved,"face":options.face}))?);
    let new_catalog = archive::parse(&patched_exe, &archives["DISK1.DAT"], &archives["DISK2.DAT"])?;
    for r in &new_catalog.resources {
        files.insert(r.path.clone(), r.bytes.clone());
    }
    let report = value!({"schema":"kohaku-import-v1","tool_version":env!("CARGO_PKG_VERSION"),"selected_json_files":picked.len(),"changed_entries":report_entries.len(),"changes":report_entries,"font_patched_glyphs":font_build.patched_glyphs,"original_font_sha256":fivec_new::sha256(&font),"font_sha256":fivec_new::sha256(&files["font.tmp"]),"images":images.iter().enumerate().map(|(i,b)|value!({"path":format!("images/disk{}.fdi",i+1),"sha256":fivec_new::sha256(b),"byte_identical_to_source":b==&game.images[i]})).collect::<Vec<_>>(),"references":new_catalog.references,"resources":new_catalog.resources,"validation":"All rebuilt FDI files and DAT resources reread through fivec-new; translated JIS decoded through shared font plan and compared to message. No emulator playthrough.","ui_images":"unchanged"});
    files.insert("import_report.json".into(), json(&report)?);
    files.insert("README.md".into(),"# 汉化构建产物\n\n将 images/disk1.fdi 和 disk2.fdi 放入 PC-98 模拟器的两台软驱。将本目录 font.tmp 作为本次构建的 NP2 字库使用；它与镜像中的字槽映射配套，不要混用其他构建的字库。\n\nfiles 是完整盘内文件，resources 是重新解包的资源，font_mapping.json 和 import_report.json 记录映射与逐条重提取验证。原盘与译文没有被覆盖。图片文字保持原样。当前完成静态结构、字库和镜像读回验证，未做全流程模拟器测试。\n".as_bytes().to_vec());
    Prepared::new(
        game.sources,
        options.output,
        files,
        catalog.resources.len(),
        picked.len(),
        report["changed_entries"].as_u64().unwrap_or(0) as usize,
    )
}
