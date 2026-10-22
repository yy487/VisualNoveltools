use crate::{Game, Result, disk, font, mes, native};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use vn_font::font_98;
use vn_text::Entry;

const MANIFEST: &str = ".lipstick-export.json";
const MARKER: &str = ".lipstick-output";
const FORMAT: &str = "lipstick-pc98-v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Script(String),
    Native(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub json: String,
    pub target: Target,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    game: Game,
    images: Vec<disk::ImageInfo>,
    bindings: Vec<Binding>,
    scripts: BTreeMap<String, mes::Primary>,
    native: BTreeMap<String, native::Primary>,
}

pub struct Job {
    pub preview: vn_cli::Preview,
    files: BTreeMap<PathBuf, Vec<u8>>,
    output: PathBuf,
    overwrite: bool,
    summary: String,
}

fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut out = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    out.push(b'\n');
    Ok(out)
}

fn relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        || path.to_string_lossy().contains(':')
    {
        return Err(format!("非法相对输出路径：{}", path.display()));
    }
    Ok(())
}

fn resolved(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path).map_err(|e| e.to_string())?;
    if absolute.exists() {
        return fs::canonicalize(absolute).map_err(|e| e.to_string());
    }
    let parent = absolute.parent().ok_or("路径缺少父目录")?;
    let parent = fs::canonicalize(parent).map_err(|e| format!("输出父目录须先存在：{e}"))?;
    Ok(parent.join(absolute.file_name().ok_or("路径缺少目录名")?))
}

fn overlaps(a: &Path, b: &Path) -> bool {
    let a = a.to_string_lossy().replace('/', "\\").to_lowercase();
    let b = b.to_string_lossy().replace('/', "\\").to_lowercase();
    a == b || a.starts_with(&(b.clone() + "\\")) || b.starts_with(&(a + "\\"))
}

impl Job {
    fn new(
        inputs: Vec<PathBuf>,
        output: &Path,
        overwrite: bool,
        mut files: BTreeMap<PathBuf, Vec<u8>>,
        summary: String,
        details: Vec<String>,
    ) -> Result<Self> {
        let output = resolved(output)?;
        let mut protected = Vec::new();
        for input in inputs {
            let input =
                fs::canonicalize(&input).map_err(|e| format!("{}: {e}", input.display()))?;
            if overlaps(&input, &output) {
                return Err("输出与原盘／提取工作区／译文／原字库重叠".into());
            }
            if !protected.contains(&input) {
                protected.push(input);
            }
        }
        if output.exists() && (!overwrite || !output.is_dir() || !output.join(MARKER).is_file()) {
            return Err("默认拒绝已有输出；overwrite 只接受本工具目录并保留整目录备份".into());
        }
        files.insert(PathBuf::from(MARKER), FORMAT.as_bytes().to_vec());
        for path in files.keys() {
            relative(path)?;
        }
        let preview = vn_cli::Preview {
            inputs: protected,
            outputs: vec![output.clone()],
            details,
            steps: vec![
                "使用本次输入快照准备全部脚本、字库和磁盘".into(),
                "写入独立暂存目录，再提交完整输出".into(),
            ],
        };
        Ok(Self {
            preview,
            files,
            output,
            overwrite,
            summary,
        })
    }

    pub fn execute(
        self,
        mut progress: impl FnMut(&str) -> Result<()>,
    ) -> Result<vn_cli::RunReport> {
        for input in &self.preview.inputs {
            if overlaps(&resolved(input)?, &resolved(&self.output)?) {
                return Err("执行时输入输出发生路径重叠".into());
            }
        }
        if self.output.exists() && (!self.overwrite || !self.output.join(MARKER).is_file()) {
            return Err("执行时出现已有输出，拒绝替换".into());
        }
        let parent = self.output.parent().ok_or("输出缺少父目录")?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let stage = parent.join(format!(".lipstick-stage-{}-{nonce}", std::process::id()));
        let backup = parent.join(format!(".lipstick-backup-{}-{nonce}", std::process::id()));
        fs::create_dir(&stage).map_err(|e| e.to_string())?;
        let result = (|| {
            progress("写入暂存目录；不做哈希校验")?;
            for (relative, bytes) in &self.files {
                let path = stage.join(relative);
                fs::create_dir_all(path.parent().ok_or("文件缺少父目录")?)
                    .map_err(|e| e.to_string())?;
                let mut f = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                f.write_all(bytes).map_err(|e| e.to_string())?;
            }
            progress("提交独立输出目录")?;
            let existed = self.output.exists();
            if existed {
                if !self.overwrite || !self.output.join(MARKER).is_file() {
                    return Err("提交时输出发生变化".into());
                }
                fs::rename(&self.output, &backup).map_err(|e| e.to_string())?;
            }
            if let Err(e) = fs::rename(&stage, &self.output) {
                if existed && let Err(restore) = fs::rename(&backup, &self.output) {
                    return Err(format!(
                        "提交失败：{e}；恢复失败：{restore}；旧输出保留于 {}",
                        backup.display()
                    ));
                }
                return Err(e.to_string());
            }
            Ok(existed)
        })();
        match result {
            Ok(existed) => Ok(vn_cli::RunReport {
                summary: if existed {
                    format!("{}；旧输出备份：{}", self.summary, backup.display())
                } else {
                    self.summary
                },
                outputs: vec![self.output],
                totals: vec![("文件数".into(), self.files.len() as u64)],
                ..Default::default()
            }),
            Err(e) => {
                if stage.exists()
                    && stage.parent() == Some(parent)
                    && stage
                        .file_name()
                        .is_some_and(|n| n.to_string_lossy().starts_with(".lipstick-stage-"))
                {
                    fs::remove_dir_all(&stage).map_err(|cleanup| {
                        format!("{e}；暂存保留于 {}：{cleanup}", stage.display())
                    })?;
                }
                Err(e)
            }
        }
    }
}

fn one<'a>(files: &'a BTreeMap<String, Vec<u8>>, name: &str) -> Result<(&'a str, &'a [u8])> {
    let found = files
        .iter()
        .filter(|(p, _)| {
            p.rsplit('/')
                .next()
                .is_some_and(|s| s.eq_ignore_ascii_case(name))
        })
        .collect::<Vec<_>>();
    if found.len() != 1 {
        return Err(format!("需要唯一的 {name}，实际 {} 个", found.len()));
    }
    Ok((found[0].0.as_str(), found[0].1.as_slice()))
}

fn parse_scripts(
    game: Game,
    files: &BTreeMap<String, Vec<u8>>,
    views: &mut BTreeMap<String, mes::View>,
) -> Result<BTreeMap<String, mes::Primary>> {
    let (_, name_def) = one(files, "NAME.DEF")?;
    let player = if game == Game::Lipstick3 {
        mes::direct("＊＊＊").ok_or("玩家名标记编码")?
    } else {
        let end = name_def
            .windows(2)
            .position(|s| s == [255, 255])
            .ok_or("NAME.DEF 缺少匹配名终止符")?;
        name_def[..end].to_vec()
    };
    let common = if game == Game::Lipstick3 {
        let (path, b) = one(files, "SC00.MES")?;
        mes::common_macros(path, b)?
    } else {
        mes::Macros::new()
    };
    let mut out = BTreeMap::new();
    for (path, b) in files {
        if Path::new(path)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("mes"))
            || (game == Game::Lipstick3
                && path
                    .rsplit('/')
                    .next()
                    .is_some_and(|s| s.eq_ignore_ascii_case("LIP3.INI")))
        {
            let (primary, view) = mes::parse_with_view(game, path, b, &common, &player)?;
            out.insert(path.clone(), primary);
            views.insert(path.clone(), view);
        }
    }
    if out.is_empty() {
        return Err("没有解析到本代脚本".into());
    }
    Ok(out)
}

fn flat_name(path: &str, native: bool, used: &mut BTreeSet<String>) -> Result<String> {
    let file = Path::new(path)
        .file_name()
        .and_then(|p| p.to_str())
        .ok_or("资源缺少文件名")?;
    let stem = if native {
        format!("exe_{}", file.replace('.', "_"))
    } else {
        Path::new(file)
            .file_stem()
            .and_then(|p| p.to_str())
            .ok_or("脚本缺少名称")?
            .to_owned()
    };
    let mut name = format!("{stem}.json");
    if !used.insert(name.to_lowercase()) {
        name = format!("{}__{stem}.json", path.split('/').next().ok_or("盘号缺失")?);
        if !used.insert(name.to_lowercase()) {
            return Err("扁平化文件名存在歧义".into());
        }
    }
    relative(Path::new(&name))?;
    Ok(name)
}

pub fn prepare_export(game: Game, source: &Path, output: &Path, overwrite: bool) -> Result<Job> {
    let paths = disk::discover(source)?;
    let disks = disk::load(game, &paths, None)?;
    let originals = disk::all_files(&disks.units);
    let mut views = BTreeMap::new();
    let scripts = parse_scripts(game, &originals, &mut views)?;
    drop(views);
    let native = native::parse(game, &originals)?
        .into_iter()
        .map(|p| (p.path.clone(), p))
        .collect::<BTreeMap<_, _>>();
    let mut files = BTreeMap::new();
    for (key, b) in originals {
        files.insert(Path::new("original").join(key), b);
    }
    for image in &disks.images {
        files.insert(
            Path::new("images").join(&image.info.name),
            image.bytes.clone(),
        );
    }
    let mut used = BTreeSet::new();
    let mut bindings = Vec::new();
    let mut count = 0;
    for (key, p) in &scripts {
        let name = flat_name(key, false, &mut used)?;
        let entries = p
            .records
            .iter()
            .map(|r| r.entry.clone())
            .collect::<Vec<_>>();
        count += entries.len();
        files.insert(
            Path::new("translations").join(&name),
            vn_text::write_template(&entries).map_err(|e| e.to_string())?,
        );
        bindings.push(Binding {
            json: name,
            target: Target::Script(key.clone()),
        });
    }
    let mut native_count = 0;
    for (key, p) in &native {
        let name = flat_name(key, true, &mut used)?;
        let entries = p
            .records
            .iter()
            .map(|r| r.entry.clone())
            .collect::<Vec<_>>();
        native_count += entries.len();
        files.insert(
            Path::new("translations").join(&name),
            vn_text::write_template(&entries).map_err(|e| e.to_string())?,
        );
        bindings.push(Binding {
            json: name,
            target: Target::Native(key.clone()),
        });
    }
    let diagnostics = scripts
        .iter()
        .flat_map(|(p, s)| s.diagnostics.iter().map(move |d| format!("{p}: {d}")))
        .collect::<Vec<_>>();
    files.insert(PathBuf::from("export_report.json"), json(&serde_json::json!({"game":game.id(),"scripts":scripts.len(),
        "script_entries":count,"native_entries":native_count,"flat_json":bindings.len(),"hash_checks":false,"diagnostics":diagnostics}))?);
    let script_count = scripts.len();
    let binding_count = bindings.len();
    files.insert(
        PathBuf::from(MANIFEST),
        json(&Manifest {
            format: FORMAT.into(),
            game,
            images: disks.images.iter().map(|i| i.info.clone()).collect(),
            bindings,
            scripts,
            native,
        })?,
    );
    let mut inputs = vec![source.to_path_buf()];
    inputs.extend(paths);
    Job::new(
        inputs,
        output,
        overwrite,
        files,
        format!(
            "{} 导出完成：{script_count} 个脚本、{count} 条脚本记录、{native_count} 条内嵌语句",
            game.id()
        ),
        vec![
            format!("{binding_count} 份平铺 JSON；每条只含 message 与可选 name"),
            "完整字节及姓名／宏来源放入独立一次提取清单；不做哈希校验".into(),
        ],
    )
}

fn collect_json(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let meta = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.file_type().is_symlink() {
        return Err("译文路径不能是符号链接".into());
    }
    if meta.is_file() {
        if path
            .extension()
            .is_none_or(|e| !e.eq_ignore_ascii_case("json"))
        {
            return Err("译文必须为 JSON".into());
        }
        out.push(path.into());
    } else if meta.is_dir() {
        if path.join("translations").is_dir() {
            return collect_json(&path.join("translations"), out);
        }
        for e in fs::read_dir(path).map_err(|e| e.to_string())? {
            let p = e.map_err(|e| e.to_string())?.path();
            if p.is_dir()
                || p.extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("json"))
            {
                collect_json(&p, out)?;
            }
        }
    } else {
        return Err("译文不是普通文件或目录".into());
    }
    Ok(())
}

pub fn prepare_import(
    source: &Path,
    translations: &[PathBuf],
    output: &Path,
    overwrite: bool,
    font_source: Option<&Path>,
    face: &str,
) -> Result<Job> {
    let manifest: Manifest =
        serde_json::from_slice(&read(&source.join(MANIFEST))?).map_err(|e| e.to_string())?;
    if manifest.format != FORMAT {
        return Err("不是本工具的提取工作区".into());
    }
    for image in &manifest.images {
        relative(Path::new(&image.name))?;
    }
    let image_paths = manifest
        .images
        .iter()
        .map(|i| source.join("images").join(&i.name))
        .collect::<Vec<_>>();
    let disks = disk::load(manifest.game, &image_paths, Some(&manifest.images))?;
    let mut originals = disk::all_files(&disks.units);
    let mut views = BTreeMap::new();
    let scripts = parse_scripts(manifest.game, &originals, &mut views)?;
    let native = native::parse(manifest.game, &originals)?
        .into_iter()
        .map(|p| (p.path.clone(), p))
        .collect::<BTreeMap<_, _>>();
    if scripts != manifest.scripts || native != manifest.native {
        return Err("当前原盘来源／一次提取记录不匹配；不做哈希比对，按原始字节和结构绑定".into());
    }
    let mut used = BTreeSet::new();
    let mut expected_bindings = Vec::new();
    for key in scripts.keys() {
        expected_bindings.push(Binding {
            json: flat_name(key, false, &mut used)?,
            target: Target::Script(key.clone()),
        });
    }
    for key in native.keys() {
        expected_bindings.push(Binding {
            json: flat_name(key, true, &mut used)?,
            target: Target::Native(key.clone()),
        });
    }
    if manifest.bindings != expected_bindings {
        return Err("翻译文件与原始资源的绑定已被更改".into());
    }
    let names = manifest
        .bindings
        .iter()
        .map(|b| (b.json.to_lowercase(), b.target.clone()))
        .collect::<BTreeMap<_, _>>();
    if names.len() != manifest.bindings.len() {
        return Err("翻译文件映射重复".into());
    }
    let selected = if translations.is_empty() {
        vec![source.join("translations")]
    } else {
        translations.to_vec()
    };
    let mut paths = Vec::new();
    for p in &selected {
        collect_json(p, &mut paths)?;
    }
    paths.sort();
    if paths.is_empty() {
        return Err("没有找到翻译 JSON".into());
    }
    let mut supplied = BTreeMap::<String, Vec<Entry>>::new();
    for p in &paths {
        let name = p
            .file_name()
            .and_then(|v| v.to_str())
            .ok_or("译文文件名不是 Unicode")?
            .to_lowercase();
        if !names.contains_key(&name) {
            return Err(format!("{} 不属于这个工作区", p.display()));
        }
        let entries =
            vn_text::read_template(&read(p)?).map_err(|e| format!("{}: {e}", p.display()))?;
        if supplied.insert(name, entries).is_some() {
            return Err("重复选择同一个译文文件".into());
        }
    }
    let mut script_edits = scripts
        .iter()
        .map(|(k, p)| {
            (
                k.clone(),
                p.records
                    .iter()
                    .map(|r| r.entry.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut native_edits = native
        .iter()
        .map(|(k, p)| {
            (
                k.clone(),
                p.records
                    .iter()
                    .map(|r| r.entry.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for (name, entries) in supplied {
        match names.get(&name).ok_or("翻译映射缺失")? {
            Target::Script(key) => {
                if !scripts.contains_key(key) {
                    return Err("非法脚本绑定".into());
                }
                script_edits.insert(key.clone(), entries);
            }
            Target::Native(key) => {
                if !native.contains_key(key) {
                    return Err("非法程序绑定".into());
                }
                native_edits.insert(key.clone(), entries);
            }
        }
    }
    let mut texts = Vec::new();
    let mut changed_entries = 0;
    let mut reserved = BTreeSet::new();
    for (key, p) in &scripts {
        texts.extend(mes::changed_texts(p, &script_edits[key])?);
        reserved.extend(font::reserved_codes(&views[key].bytes));
        changed_entries += p
            .records
            .iter()
            .zip(&script_edits[key])
            .filter(|(r, e)| r.entry != **e)
            .count();
    }
    for (key, p) in &native {
        texts.extend(native::changed_texts(p, &native_edits[key])?);
        changed_entries += p
            .records
            .iter()
            .zip(&native_edits[key])
            .filter(|(r, e)| r.entry != **e)
            .count();
    }
    for (key, data) in &originals {
        let ext = Path::new(key)
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            "exe" | "ovl" | "ov1" if data.starts_with(b"MZ") => {
                reserved.extend(font::reserved_codes(&native::module(data)?.runtime))
            }
            "com" | "sys" | "def" => reserved.extend(font::reserved_codes(data)),
            _ => {}
        }
    }
    // Keep the native symbol/kana pages used by compact text and BIOS routines.
    for code in 0x8140u16..=0x879c {
        reserved.extend(font::reserved_codes(&code.to_be_bytes()));
    }
    let original_font = match font_source {
        Some(p) => read(p)?,
        None => font_98::EMBEDDED_FONT.to_vec(),
    };
    let font = font::Font::prepare(manifest.game, &original_font, &reserved, &texts, face)?;
    let encode = |s: &str, ascii: bool| font.encode(s, ascii);
    let mut replacements = BTreeMap::new();
    let mut cache_scripts = BTreeSet::new();
    for view in views.values() {
        for t in &view.tokens {
            if manifest.game == Game::Lipstick3
                && let mes::Kind::Op(0xc0, args) = &t.kind
                && let Some(mes::Arg::String(b)) = args.first()
            {
                let path = mes::decode(b)?.replace('\\', "/");
                cache_scripts.insert(path.rsplit('/').next().unwrap_or(&path).to_lowercase());
            }
        }
    }
    for (key, p) in &scripts {
        let bytes = mes::rebuild_view(p, &views[key], &script_edits[key], &encode)
            .map_err(|e| format!("{key}: {e}"))?;
        let name = key.rsplit('/').next().unwrap_or(key).to_lowercase();
        let limit = match manifest.game {
            Game::Lipstick1 => 0x8000,
            Game::Lipstick2 => 0x8000,
            Game::Lipstick3 if cache_scripts.contains(&name) => 0x1000,
            Game::Lipstick3 => 0x3000,
        };
        if bytes.len() > limit && bytes != p.original {
            return Err(format!(
                "{key}: 回注后 {} 字节超过引擎区域 {limit} 字节",
                bytes.len()
            ));
        }
        replacements.insert(key.clone(), bytes);
    }
    for (key, p) in &native {
        replacements.insert(
            key.clone(),
            native::rebuild(p, &native_edits[key], &encode)?,
        );
    }
    let rebuilt = disk::rebuild(&disks, &replacements)?;
    originals.extend(replacements);
    originals.extend(rebuilt.containers);
    let mut files = BTreeMap::new();
    for (key, b) in originals {
        files.insert(Path::new("files").join(key), b);
    }
    for (name, b) in rebuilt.images {
        files.insert(Path::new("images").join(name), b);
    }
    files.insert(PathBuf::from("font.bmp"), font.bytes);
    files.insert(PathBuf::from("font_mapping.json"), json(&font.mapping)?);
    files.insert(PathBuf::from("build_report.json"), json(&serde_json::json!({"game":manifest.game.id(),"changed_entries":changed_entries,
        "patched_glyphs":font.patched_glyphs,"hash_checks":false,"custom_font_resources":"preserved",
        "native_wrappers":"unchanged programs preserved; edited packed programs use EXEPACK without outer TC0/XOR"}))?);
    let mut inputs = vec![source.to_path_buf()];
    inputs.extend(selected);
    if let Some(p) = font_source {
        inputs.push(p.into());
    }
    Job::new(
        inputs,
        output,
        overwrite,
        files,
        format!(
            "{} 回注完成：修改 {changed_entries} 条，生成完整 FDI 与共用 NP2 字库",
            manifest.game.id()
        ),
        vec![
            "保留 KANJI.FNT / G16 / NAME.DEF 与原生字形占位符".into(),
            "在写出前检查槽位、脚本缓冲和软盘容量；不做哈希校验".into(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flat_names_are_unique_and_cannot_escape_the_output() {
        let mut used = BTreeSet::new();
        assert_eq!(
            flat_name("disk_a/MES/X.MES", false, &mut used).unwrap(),
            "X.json"
        );
        assert_eq!(
            flat_name("disk_b/MES/X.MES", false, &mut used).unwrap(),
            "disk_b__X.json"
        );
        assert!(relative(Path::new("../outside")).is_err());
        assert!(relative(Path::new("C:/outside")).is_err());
        assert!(relative(&Path::new("images").join("disk.fdi")).is_ok());
    }
}
