//! Thin workflow examples. Games can use font_88 / font_98 directly or register
//! these standalone build operations in their vn-cli panel.
use crate::{font_88, font_98};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use vn_cli::*;

/// A build request produced by the game adapter after merging selected JSON
/// translations into its full original baseline. These are display strings,
/// not raw script control syntax. This is not a replacement translation format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FontRequest {
    pub schema_version: u32,
    pub backend: String,
    pub final_texts: Vec<String>,
    /// Union of all original CP932 double-byte slots that must remain unchanged.
    pub reserved_cp932: Vec<u16>,
}

pub fn operations() -> Vec<Box<dyn Operation>> {
    vec![
        Box::new(FontOperation::pc88()),
        Box::new(FontOperation::pc98()),
    ]
}

pub struct FontOperation {
    pc88: bool,
}
impl FontOperation {
    pub fn pc88() -> Self {
        Self { pc88: true }
    }
    pub fn pc98() -> Self {
        Self { pc88: false }
    }
    fn backend(&self) -> &'static str {
        if self.pc88 {
            "pc88-kanji1"
        } else {
            "pc98-np2"
        }
    }
}
impl Operation for FontOperation {
    fn spec(&self) -> OperationSpec {
        let mut fields = vec![
            Field::new("source", "原始字库", FieldKind::Path).required(),
            Field::new("request", "完整构建请求 JSON", FieldKind::Path).required(),
            Field::new("output", "独立输出目录", FieldKind::Path).required(),
            Field::new(
                "mapping",
                "替换候选表（未设置使用内置资源）",
                FieldKind::Path,
            ),
        ];
        if self.pc88 {
            fields.push(Field::new(
                "glyphs",
                "FCG1 点阵表（未设置使用内置资源）",
                FieldKind::Path,
            ));
        } else {
            fields.push(
                Field::new("face", "绘字字体", FieldKind::Text)
                    .required()
                    .default(Value::Text(font_98::FONT_FACE.into())),
            );
        }
        let mut spec = OperationSpec::new(
            if self.pc88 { "build-88" } else { "build-98" },
            if self.pc88 {
                "生成 PC88 KANJI1 字库"
            } else {
                "生成 PC98 NP2 字库"
            },
            fields,
        );
        spec.description =
            "读取原字库与游戏适配层生成的构建请求，输出字库、映射及校验记录。".into();
        spec
    }
    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        let mut sources = Vec::new();
        let mut requests = Vec::new();
        for path in paths {
            if !path.is_file() {
                continue;
            }
            let metadata = fs::metadata(path)?;
            if metadata.len() > 32 * 1024 * 1024 {
                continue;
            }
            let bytes = fs::read(path)?;
            let font_ok = if self.pc88 {
                font_88::validate_rom(&bytes).is_ok()
            } else {
                font_98::validate_font(&bytes).is_ok()
            };
            if font_ok {
                sources.push(path);
            }
            if serde_json::from_slice::<FontRequest>(&bytes)
                .is_ok_and(|r| r.schema_version == 1 && r.backend == self.backend())
            {
                requests.push(path);
            }
        }
        if sources.len() > 1 || requests.len() > 1 {
            return Err("有多个候选输入，请在参数页明确选择".into());
        }
        if let Some(path) = sources.first() {
            parameters.set("source", Value::Path((*path).clone()));
        }
        if let Some(path) = requests.first() {
            parameters.set("request", Value::Path((*path).clone()));
        }
        Ok(())
    }
    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source_path = fs::canonicalize(parameters.path("source")?)?;
        let request_path = fs::canonicalize(parameters.path("request")?)?;
        let source = fs::read(&source_path)?;
        let request_bytes = fs::read(&request_path)?;
        let request: FontRequest = serde_json::from_slice(&request_bytes)
            .map_err(|e| Error(format!("构建请求 {}: {e}", request_path.display())))?;
        if request.schema_version != 1 || request.backend != self.backend() {
            return Err("构建请求版本或字库类型不匹配".into());
        }
        let mut inputs = vec![source_path, request_path];
        let mut resource_hashes = BTreeMap::new();
        let mapping = match parameters.get("mapping") {
            Some(Value::Path(path)) => {
                let bytes = fs::read(path)?;
                inputs.push(fs::canonicalize(path)?);
                bytes
            }
            _ => {
                if self.pc88 {
                    font_88::EMBEDDED_MAPPING.to_vec()
                } else {
                    font_98::EMBEDDED_SUBSTITUTIONS.as_bytes().to_vec()
                }
            }
        };
        resource_hashes.insert("mapping", digest(&mapping));
        let mut files = BTreeMap::new();
        let patched;
        if self.pc88 {
            let glyphs = match parameters.get("glyphs") {
                Some(Value::Path(path)) => {
                    let bytes = fs::read(path)?;
                    inputs.push(fs::canonicalize(path)?);
                    bytes
                }
                _ => font_88::EMBEDDED_GLYPHS.to_vec(),
            };
            resource_hashes.insert("glyphs", digest(&glyphs));
            let resources = font_88::FontResources::from_bytes(&mapping, &glyphs).map_err(Error)?;
            let plan = resources
                .plan_dynamic_mapping(request.reserved_cp932.iter().copied(), &request.final_texts)
                .map_err(Error)?;
            // Verify every string with the exact same plan used to build the ROM.
            for text in &request.final_texts {
                resources.encode_text(text, &plan).map_err(Error)?;
            }
            let built = resources.build_rom(&source, &plan).map_err(Error)?;
            patched = built.manifest.patched_slots;
            for page in resources
                .render_preview_pages(&built.rom, &plan, &font_88::PreviewOptions::default())
                .map_err(Error)?
            {
                files.insert(page.file_name, page.bmp);
            }
            files.insert("font_plan.json".into(), json(&plan)?);
            files.insert(
                "mapping_used.json".into(),
                font_88::mapping_used_json_bytes(&plan).map_err(Error)?,
            );
            files.insert(
                "backend_manifest.json".into(),
                font_88::manifest_json_bytes(&built.manifest).map_err(Error)?,
            );
            files.insert("KANJI1.ROM".into(), built.rom);
        } else {
            let resources = font_98::SubstitutionMap::from_json(&mapping).map_err(Error)?;
            let reserved = request
                .reserved_cp932
                .iter()
                .copied()
                .collect::<BTreeSet<_>>();
            let plan = font_98::EncodingPlan::build(
                &resources,
                reserved.iter().copied(),
                request.final_texts.iter().map(String::as_str),
            )
            .map_err(Error)?;
            for text in &request.final_texts {
                plan.encode_cp932(text).map_err(Error)?;
            }
            let built = font_98::prepare_font(
                &source,
                &plan.requests(),
                &reserved,
                parameters.text("face")?,
            )
            .map_err(Error)?;
            patched = built.patched_glyphs;
            files.insert(
                "mapping_used.json".into(),
                json(&plan.manifest_entries().map_err(Error)?)?,
            );
            files.insert("font.tmp".into(), built.bytes);
        }
        files.insert("font_request.json".into(), json(&request)?);
        let hashes = files
            .iter()
            .map(|(name, bytes)| (name.clone(), digest(bytes)))
            .collect::<BTreeMap<_, _>>();
        files.insert("font_manifest.json".into(), json(&serde_json::json!({
            "schema_version": 1, "tool_version": env!("CARGO_PKG_VERSION"), "backend": self.backend(),
            "source_sha256": digest(&source), "request_sha256": digest(&request_bytes),
            "resources_sha256": resource_hashes, "outputs_sha256": hashes,
            "final_text_count": request.final_texts.len(), "reserved_cp932": request.reserved_cp932,
            "patched_glyphs": patched, "non_target_bytes_preserved": true,
            "font_face": parameters.get("face").and_then(|v| if let Value::Text(face) = v { Some(face.as_str()) } else { None }),
        }))?);
        let output = absolute_output(parameters.path("output")?)?;
        let previous = snapshot(&output)?;
        if previous.is_some() && !parameters.flag("overwrite") {
            return Err("输出目录已存在，默认不覆盖".into());
        }
        let preview = Preview {
            inputs,
            outputs: vec![output.clone()],
            steps: vec![
                "校验完整文本与保留字槽".into(),
                "生成字库并校验修改范围".into(),
                "提交字库、映射与构建记录".into(),
            ],
            details: vec![
                format!("文本数: {}，重绘字槽: {patched}", request.final_texts.len()),
                format!(
                    "将生成: {}",
                    files.keys().cloned().collect::<Vec<_>>().join("、")
                ),
            ],
        };
        Ok(Box::new(FontJob {
            preview,
            output,
            previous,
            files,
            patched,
        }))
    }
}
fn json(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| Error(e.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn absolute_output(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    let parent = fs::canonicalize(absolute.parent().ok_or("输出目录缺少父目录")?)?;
    Ok(parent.join(absolute.file_name().ok_or("需要独立的输出目录名")?))
}
type Snapshot = BTreeMap<PathBuf, Option<String>>;
fn snapshot(root: &Path) -> Result<Option<Snapshot>> {
    if !root.try_exists()? {
        return Ok(None);
    }
    if !root.is_dir() {
        return Err("输出已存在且不是目录".into());
    }
    fn walk(root: &Path, path: &Path, result: &mut Snapshot) -> Result<()> {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() {
            return Err("输出树包含链接，请使用新的独立目录".into());
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|e| Error(e.to_string()))?
            .to_path_buf();
        if metadata.is_dir() {
            result.insert(relative, None);
            for entry in fs::read_dir(path)? {
                walk(root, &entry?.path(), result)?;
            }
        } else if metadata.is_file() {
            result.insert(relative, Some(digest(&fs::read(path)?)));
        } else {
            return Err("输出树包含不支持的文件类型".into());
        }
        Ok(())
    }
    let mut result = Snapshot::new();
    walk(root, root, &mut result)?;
    Ok(Some(result))
}
struct OwnedDirectory(PathBuf);
impl OwnedDirectory {
    fn create(parent: &Path) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let path = parent.join(format!(
                ".vn-font-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Err("无法分配本次构建临时目录".into())
    }
}
impl Drop for OwnedDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct FontJob {
    preview: Preview,
    output: PathBuf,
    previous: Option<Snapshot>,
    files: BTreeMap<String, Vec<u8>>,
    patched: usize,
}
impl PreparedOperation for FontJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }
    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("字库已在内存中通过校验，正在写入本次构建目录")?;
        let stage = OwnedDirectory::create(self.output.parent().ok_or("输出缺少父目录")?)?;
        for (name, bytes) in &self.files {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(stage.0.join(name))?;
            file.write_all(bytes)?;
            file.sync_all()?;
        }
        if snapshot(&self.output)? != self.previous {
            return Err("输出在预检后发生变化，请重新预检".into());
        }
        let mut backup = None;
        if self.previous.is_some() {
            let holder = OwnedDirectory::create(self.output.parent().ok_or("输出缺少父目录")?)?;
            let old = holder.0.join("previous");
            fs::rename(&self.output, &old)?;
            backup = Some((holder, old));
        }
        if let Err(error) = fs::rename(&stage.0, &self.output) {
            if let Some((holder, old)) = backup {
                if let Err(restore_error) = fs::rename(&old, &self.output) {
                    let saved = holder.0.clone();
                    std::mem::forget(holder);
                    return Err(format!(
                        "提交失败: {error}；恢复失败: {restore_error}；原输出保留于 {}",
                        saved.display()
                    )
                    .into());
                }
            }
            return Err(error.into());
        }
        let mut warnings = Vec::new();
        if let Some((holder, _)) = backup {
            if let Err(error) = fs::remove_dir_all(&holder.0) {
                warnings.push(format!(
                    "新输出已提交，旧输出清理未完成: {error}；残留位置 {}",
                    holder.0.display()
                ));
                // Preserve any remaining backup instead of silently retrying in Drop.
                std::mem::forget(holder);
            }
        }
        Ok(RunReport {
            summary: "字库及配套记录生成完成".into(),
            totals: vec![
                ("重绘字槽".into(), self.patched as u64),
                ("输出文件".into(), self.files.len() as u64),
            ],
            outputs: vec![self.output.clone()],
            warnings,
        })
    }
}
