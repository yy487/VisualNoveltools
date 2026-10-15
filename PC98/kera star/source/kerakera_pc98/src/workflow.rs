use crate::{disk, mes, runtime};
use encoding_rs::SHIFT_JIS;
use mes::{Primary, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use vn_font::font_98::{self, EncodingPlan, SubstitutionMap};
use vn_text::Entry;

const MANIFEST: &str = ".kerakera-export.json";
const SCHEMA: &str = "kerakera-pc98-export-v1";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    disks: Vec<String>,
    scripts: BTreeMap<String, Primary>,
}

pub struct Job {
    pub preview: vn_cli::Preview,
    files: BTreeMap<PathBuf, Vec<u8>>,
    output: PathBuf,
    overwrite: bool,
    summary: String,
}

fn json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty(value)
        .map(|mut b| {
            b.push(b'\n');
            b
        })
        .map_err(|e| e.to_string())
}

fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn safe_relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("不是安全相对路径: {}", path.display()));
    }
    Ok(())
}

fn resolved(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path).map_err(|e| e.to_string())?;
    if absolute.exists() {
        return fs::canonicalize(absolute).map_err(|e| e.to_string());
    }
    let parent = absolute.parent().ok_or("输出缺少父目录")?;
    let parent = fs::canonicalize(parent).map_err(|e| format!("输出父目录须已存在: {e}"))?;
    Ok(parent.join(absolute.file_name().ok_or("输出缺少目录名称")?))
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
        files: BTreeMap<PathBuf, Vec<u8>>,
        summary: String,
    ) -> Result<Self> {
        let output = resolved(output)?;
        let mut canonical = Vec::new();
        for input in inputs {
            let input =
                fs::canonicalize(&input).map_err(|e| format!("{}: {e}", input.display()))?;
            if overlaps(&input, &output) {
                return Err("输出与受保护的原始数据/译文路径重叠".into());
            }
            canonical.push(input);
        }
        if output.exists() {
            if !overwrite {
                return Err("输出已存在；默认拒绝覆盖".into());
            }
            if !output.is_dir() || !output.join(".kerakera-output").is_file() {
                return Err("仅允许覆盖本工具创建的输出目录".into());
            }
        }
        for path in files.keys() {
            safe_relative(path)?;
        }
        let preview = vn_cli::Preview {
            inputs: canonical,
            outputs: vec![output.clone()],
            details: vec![
                summary.clone(),
                format!("写出 {} 个文件；原始镜像和译文均保留", files.len()),
            ],
            steps: vec![
                "解析本作脚本与姓名片段，校验译文".into(),
                "准备脚本、字库和磁盘内容".into(),
                "暂存全部文件后提交到独立目录".into(),
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
        // Library callers receive the same path protection as CLI callers.
        for input in &self.preview.inputs {
            if overlaps(&resolved(input)?, &resolved(&self.output)?) {
                return Err("执行时输出与输入重叠".into());
            }
        }
        if self.output.exists()
            && (!self.overwrite || !self.output.join(".kerakera-output").is_file())
        {
            return Err("输出已出现或不是本工具目录，拒绝替换".into());
        }
        let parent = self.output.parent().ok_or("输出缺少父目录")?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let stage = parent.join(format!(".kerakera-stage-{}-{nonce}", std::process::id()));
        let backup = parent.join(format!(".kerakera-backup-{}-{nonce}", std::process::id()));
        fs::create_dir(&stage).map_err(|e| e.to_string())?;
        let write_result = (|| {
            progress("写入暂存目录")?;
            for (relative, bytes) in &self.files {
                safe_relative(relative)?;
                let path = stage.join(relative);
                fs::create_dir_all(path.parent().ok_or("输出文件缺少父目录")?)
                    .map_err(|e| e.to_string())?;
                let mut f = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                f.write_all(bytes).map_err(|e| e.to_string())?;
                f.sync_all().map_err(|e| e.to_string())?;
            }
            fs::write(stage.join(".kerakera-output"), b"kerakera-pc98-v1\n")
                .map_err(|e| e.to_string())?;
            progress("提交独立输出目录")?;
            let existed = self.output.exists();
            if existed {
                if !self.overwrite || !self.output.join(".kerakera-output").is_file() {
                    return Err("提交时拒绝覆盖输出".into());
                }
                fs::rename(&self.output, &backup).map_err(|e| e.to_string())?;
            }
            if let Err(error) = fs::rename(&stage, &self.output) {
                if existed && let Err(restore) = fs::rename(&backup, &self.output) {
                    return Err(format!(
                        "提交失败: {error}；恢复失败: {restore}；旧输出保留在 {}",
                        backup.display()
                    ));
                }
                return Err(error.to_string());
            }
            Ok(existed)
        })();
        match write_result {
            Ok(existed) => Ok(vn_cli::RunReport {
                summary: if existed {
                    format!("{}；旧输出备份: {}", self.summary, backup.display())
                } else {
                    self.summary
                },
                outputs: vec![self.output],
                totals: vec![("文件数".into(), self.files.len() as u64)],
                ..vn_cli::RunReport::default()
            }),
            Err(error) => {
                if stage.exists()
                    && let Err(cleanup) = fs::remove_dir_all(&stage)
                {
                    return Err(format!(
                        "{error}；暂存清理失败: {cleanup}；保留 {}",
                        stage.display()
                    ));
                }
                Err(error)
            }
        }
    }
}

fn game_files(disks: &[disk::Disk]) -> BTreeMap<String, Vec<u8>> {
    disks
        .iter()
        .flat_map(|d| d.files.iter().map(|(p, b)| (disk::key(d, p), b.clone())))
        .collect()
}

fn game_file<'a>(files: &'a BTreeMap<String, Vec<u8>>, name: &str) -> Result<&'a [u8]> {
    let matches = files
        .iter()
        .filter(|(p, _)| {
            p.rsplit('/')
                .next()
                .is_some_and(|p| p.eq_ignore_ascii_case(name))
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(format!("应有一个 {name}，实际找到 {} 个", matches.len()));
    }
    Ok(matches[0].1)
}

fn primary(files: &BTreeMap<String, Vec<u8>>) -> Result<BTreeMap<String, Primary>> {
    runtime::validate_engine(game_file(files, "ADV98V.OVL")?)?;
    runtime::ini(game_file(files, "ADV98V.INI")?)?;
    let common = mes::startup_macros(game_file(files, "KER_STUP.MES")?)?;
    let mut out = BTreeMap::new();
    for (name, bytes) in files {
        if Path::new(name)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("mes"))
        {
            out.insert(name.clone(), mes::parse(name, bytes, &common)?);
        }
    }
    if out.is_empty() {
        return Err("镜像没有本作 MES".into());
    }
    Ok(out)
}

fn translation_path(key: &str) -> PathBuf {
    Path::new(key).with_extension("json")
}

pub fn prepare_export(source: &Path, output: &Path, overwrite: bool) -> Result<Job> {
    let paths = disk::discover(source)?;
    let disks = disk::load_paths(&paths)?;
    let originals = game_files(&disks);
    let scripts = primary(&originals)?;
    let count: usize = scripts.values().map(|p| p.records.len()).sum();
    let mut files = BTreeMap::new();
    for (key, bytes) in originals {
        files.insert(Path::new("original").join(key), bytes);
    }
    for disk in &disks {
        files.insert(Path::new("images").join(&disk.name), disk.bytes.clone());
    }
    for (key, p) in &scripts {
        let entries = p
            .records
            .iter()
            .map(|r| r.entry.clone())
            .collect::<Vec<_>>();
        files.insert(
            Path::new("translations").join(translation_path(key)),
            vn_text::write_template(&entries).map_err(|e| e.to_string())?,
        );
    }
    files.insert(
        PathBuf::from(MANIFEST),
        json(&Manifest {
            format: SCHEMA.into(),
            disks: disks.iter().map(|d| d.name.clone()).collect(),
            scripts,
        })?,
    );
    let mut inputs = vec![source.to_path_buf()];
    inputs.extend(paths);
    Job::new(
        inputs,
        output,
        overwrite,
        files,
        format!("提取完成：{count} 条 name/message 记录"),
    )
}

fn json_files(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() {
        return Err("译文路径不能是符号链接".into());
    }
    if meta.is_file() {
        if !path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("json"))
        {
            return Err("译文必须是 JSON".into());
        }
        out.push(path.to_path_buf());
    } else if meta.is_dir() {
        let inner = path.join("translations");
        if inner.is_dir() {
            return json_files(&inner, out);
        }
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let p = entry.map_err(|e| e.to_string())?.path();
            if p.is_dir()
                || p.extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("json"))
            {
                json_files(&p, out)?;
            }
        }
    } else {
        return Err("译文路径不是文件或目录".into());
    }
    Ok(())
}

pub fn prepare_import(
    source: &Path,
    translations: &[PathBuf],
    output: &Path,
    overwrite: bool,
    font: Option<&Path>,
    face: &str,
) -> Result<Job> {
    let manifest: Manifest = serde_json::from_slice(&read(&source.join(MANIFEST))?)
        .map_err(|e| format!("提取工作区清单: {e}"))?;
    if manifest.format != SCHEMA {
        return Err("不是本工具的提取工作区".into());
    }
    let mut disk_paths = Vec::new();
    for name in &manifest.disks {
        safe_relative(Path::new(name))?;
        if Path::new(name).components().count() != 1 {
            return Err("非法镜像映射".into());
        }
        disk_paths.push(source.join("images").join(name));
    }
    let disks = disk::load_paths(&disk_paths)?;
    let originals = game_files(&disks);
    let scripts = primary(&originals)?;
    if scripts != manifest.scripts {
        return Err("原盘或一次提取记录已变更，请基于正确来源重新提取".into());
    }
    let selected = if translations.is_empty() {
        vec![source.join("translations")]
    } else {
        translations.to_vec()
    };
    let mut names = BTreeMap::new();
    for key in scripts.keys() {
        let filename = translation_path(key)
            .file_name()
            .ok_or("翻译映射缺少文件名")?
            .to_string_lossy()
            .to_lowercase();
        if names.insert(filename, key.clone()).is_some() {
            return Err("脚本同名歧义，不能绑定独立译文文件".into());
        }
    }
    let mut supplied: BTreeMap<String, Vec<Entry>> = scripts
        .iter()
        .map(|(k, p)| {
            (
                k.clone(),
                p.records.iter().map(|r| r.entry.clone()).collect(),
            )
        })
        .collect();
    let mut paths = Vec::new();
    for path in &selected {
        json_files(path, &mut paths)?;
    }
    paths.sort();
    if paths.is_empty() {
        return Err("未找到翻译 JSON".into());
    }
    let mut seen = BTreeSet::new();
    for path in &paths {
        let name = path
            .file_name()
            .ok_or("译文缺少文件名")?
            .to_string_lossy()
            .to_lowercase();
        let key = names
            .get(&name)
            .ok_or_else(|| format!("{} 不属于本次工作区", path.display()))?;
        if !seen.insert(key.clone()) {
            return Err(format!("重复选择译文: {key}"));
        }
        let entries =
            vn_text::read_template(&read(path)?).map_err(|e| format!("{}: {e}", path.display()))?;
        mes::text_changes(&scripts[key], &entries).map_err(|e| format!("{key}: {e}"))?;
        supplied.insert(key.clone(), entries);
    }
    let mut texts = Vec::new();
    let mut reserved = BTreeSet::new();
    let mut changed = 0;
    for (key, p) in &scripts {
        texts.extend(mes::text_changes(p, &supplied[key])?);
        changed += p
            .records
            .iter()
            .zip(&supplied[key])
            .filter(|(a, b)| a.entry != **b)
            .count();
        let remainder = mes::rebuild(p, &supplied[key], &|_| Ok(Vec::new()))?;
        reserved.extend(mes::reserved_codes(&remainder));
    }
    // Native routines and system messages also use the emulator font. Protect
    // every valid pair in their decoded modules, rather than guessing strings.
    for (path, data) in &originals {
        let extension = Path::new(path)
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        match extension.as_str() {
            "ovl" => {
                reserved.extend(mes::reserved_codes(&runtime::module(data)?));
            }
            "ini" => {
                reserved.extend(mes::reserved_codes(&runtime::ini(data)?));
            }
            "tcm" | "sys" | "com" => {
                reserved.extend(mes::reserved_codes(data));
            }
            _ => {}
        }
    }
    // The native renderer and compressed 2D..7F text share the symbol/kana pages.
    for code in 0x8140u16..=0x879c {
        reserved.extend(mes::reserved_codes(&code.to_be_bytes()));
    }
    let plan = EncodingPlan::build_with_forbidden_cp932(
        &SubstitutionMap::embedded()?,
        reserved.iter().copied(),
        mes::SPECIAL,
        texts.iter().map(String::as_str),
    )?;
    let original_font = match font {
        Some(path) => read(path)?,
        None => font_98::EMBEDDED_FONT.to_vec(),
    };
    let font_result = font_98::prepare_font(&original_font, &plan.requests(), &reserved, face)?;
    let encode = |text: &str| -> Result<Vec<u8>> {
        let bytes = plan.encode_cp932(text)?;
        if bytes.chunks_exact(2).any(|p| {
            !mes::is_lead(p[0]) || mes::SPECIAL.contains(&u16::from_be_bytes([p[0], p[1]]))
        }) {
            return Err("字库映射使用了不安全的引擎控制码".into());
        }
        Ok(bytes)
    };
    let mut replacements = BTreeMap::new();
    let mut sub_scripts = BTreeSet::new();
    for p in scripts.values() {
        for t in mes::lex(&p.original)? {
            if let mes::Kind::Op(0xc0, a) = t.kind
                && let Some(mes::Arg::String(name)) = a.first()
            {
                let (name, _, bad) = SHIFT_JIS.decode(name);
                if bad {
                    return Err("C0 文件名无法解码".into());
                }
                sub_scripts.insert(
                    name.replace('\\', "/")
                        .rsplit('/')
                        .next()
                        .unwrap_or_default()
                        .to_ascii_lowercase(),
                );
            }
        }
    }
    for (key, p) in &scripts {
        let bytes = mes::rebuild(p, &supplied[key], &encode)?;
        let name = key
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let limit = if sub_scripts.contains(&name) {
            0x1000
        } else {
            0x3000
        };
        if bytes.len() > limit {
            return Err(format!(
                "{key}: {} 字节超过解释缓冲上限 {limit}，请缩短译文",
                bytes.len()
            ));
        }
        if bytes != p.original {
            for t in mes::lex(&p.original)? {
                if let mes::Kind::Op(0xcd, args) = t.kind
                    && matches!(args.first(), Some(mes::Arg::Number(0x9000..=0xffff)))
                {
                    return Err(format!("{key}: 内嵌机器码地址不能在变长回注时自动重定位"));
                }
            }
            replacements.insert(key.clone(), bytes);
        }
    }
    let mut files = BTreeMap::new();
    for disk in &disks {
        let mut per_disk = BTreeMap::new();
        for (path, original) in &disk.files {
            let key = disk::key(disk, path);
            let bytes = replacements.get(&key).unwrap_or(original).clone();
            files.insert(Path::new("files").join(&key), bytes.clone());
            if bytes != *original {
                per_disk.insert(path.clone(), bytes);
            }
        }
        files.insert(
            Path::new("images").join(&disk.name),
            disk::rebuild(disk, &per_disk)?,
        );
    }
    files.insert(PathBuf::from("font.bmp"), font_result.bytes);
    files.insert(
        PathBuf::from("font_mapping.json"),
        json(&plan.manifest_entries()?)?,
    );
    files.insert(PathBuf::from("build_report.json"), json(&serde_json::json!({
        "format":"kerakera-pc98-build-v1", "selected_json":seen.len(), "changed_entries":changed,
        "changed_scripts":replacements.len(), "font_face":face, "patched_glyphs":font_result.patched_glyphs,
        "reserved_cp932_slots":reserved.len(), "name_layout":"［name］ + A5 + message",
        "newline":"message LF -> A5; window also wraps automatically", "source_images_untouched":true
    }))?);
    let mut inputs = vec![source.to_path_buf()];
    inputs.extend(selected);
    inputs.extend(disk_paths);
    inputs.extend(paths);
    if let Some(font) = font {
        inputs.push(font.to_path_buf());
    }
    Job::new(
        inputs,
        output,
        overwrite,
        files,
        format!(
            "注回完成：修改 {changed} 条记录、{} 个脚本，生成配套 NP2 字库与 FDI",
            replacements.len()
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path_binding_rejects_traversal_and_alias_components() {
        assert!(safe_relative(Path::new("kera_a/MES/KERA_00.json")).is_ok());
        assert!(safe_relative(Path::new("../KERA_00.json")).is_err());
        assert!(overlaps(
            Path::new("C:\\a\\input"),
            Path::new("c:\\A\\INPUT\\out")
        ));
        assert!(!overlaps(
            Path::new("C:\\a\\input"),
            Path::new("C:\\a\\input2")
        ));
    }
}
