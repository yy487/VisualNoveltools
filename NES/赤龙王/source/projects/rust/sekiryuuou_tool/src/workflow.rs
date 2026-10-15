//! Read-only preparation and transactional directory output, shared by the library and CLI.
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Panel, Parameters, PreparedOperation, Preview,
    Progress, RunReport, Value,
};

use crate::{
    chinese_export_plan, chinese_import_plan, cycle_plan, export_plan, import_plan, ExportPlan,
    Result, INPUT_FILES,
};

/// Owns the validated file snapshot. Its private boundary is checked again before publication.
pub struct PreparedDirectory {
    boundary: OutputBoundary,
    files: BTreeMap<String, Vec<u8>>,
    summary: String,
}

#[derive(Debug)]
pub struct DirectoryReport {
    pub output: PathBuf,
    pub file_count: usize,
    pub byte_count: u64,
    pub summary: String,
}

impl PreparedDirectory {
    fn from_plan(boundary: OutputBoundary, plan: ExportPlan) -> Result<Self> {
        validate_plan_paths(&plan.files)?;
        Ok(Self {
            boundary,
            files: plan.files,
            summary: plan.summary,
        })
    }

    /// Publishes to a new directory; every existing destination is refused, even if empty.
    pub fn commit(self) -> Result<DirectoryReport> {
        self.boundary.recheck()?;
        let parent = self.boundary.output.parent().ok_or("输出缺少父目录")?;
        let mut parents = CreatedParents::ensure(parent)?;
        self.boundary.recheck()?;
        let stage = StagingDirectory::create(parent)?;
        for (relative, bytes) in &self.files {
            let path = stage.path.join(normalize_plan_path(relative)?);
            fs::create_dir_all(path.parent().ok_or("计划文件缺少父目录")?)?;
            let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
            file.write_all(bytes)?;
            file.sync_all()?;
        }
        self.boundary.recheck()?;
        publish_new_directory(&stage.path, &self.boundary.output)?;
        parents.keep = true;
        Ok(DirectoryReport {
            output: self.boundary.output,
            file_count: self.files.len(),
            byte_count: self.files.values().map(|bytes| bytes.len() as u64).sum(),
            summary: self.summary,
        })
    }

    fn preview(&self, steps: Vec<String>) -> Preview {
        Preview {
            inputs: self
                .boundary
                .inputs
                .iter()
                .map(|input| input.requested.clone())
                .collect(),
            outputs: vec![self.boundary.requested.clone()],
            steps,
            details: vec![
                self.summary.clone(),
                format!(
                    "输出 {} 个文件，共 {} 字节；输出目录必须不存在（--overwrite 也不替换已有目录）。",
                    self.files.len(),
                    self.files.values().map(Vec::len).sum::<usize>()
                ),
            ],
        }
    }
}

pub fn prepare_export(source: &Path, output: &Path) -> Result<PreparedDirectory> {
    let boundary = OutputBoundary::prepare(&[source.to_path_buf()], output)?;
    let rom = read_file(&boundary.inputs[0].resolved)?;
    PreparedDirectory::from_plan(boundary, export_plan(&rom)?)
}

pub fn prepare_cycle(source: &Path, output: &Path) -> Result<PreparedDirectory> {
    let boundary = OutputBoundary::prepare(&[source.to_path_buf()], output)?;
    let rom = read_file(&boundary.inputs[0].resolved)?;
    PreparedDirectory::from_plan(boundary, cycle_plan(&rom)?)
}

pub fn prepare_import(workspace: &Path, output: &Path, expand: bool) -> Result<PreparedDirectory> {
    let workspace_path = fs::canonicalize(workspace)?;
    if !workspace_path.is_dir() {
        return Err(format!("导出工作区必须是目录: {}", workspace.display()).into());
    }
    let mut inputs = vec![workspace.to_path_buf()];
    inputs.extend(INPUT_FILES.iter().map(|name| workspace.join(name)));
    let boundary = OutputBoundary::prepare(&inputs, output)?;
    let files = INPUT_FILES
        .iter()
        .zip(boundary.inputs.iter().skip(1))
        .map(|(name, input)| Ok(((*name).to_owned(), read_file(&input.resolved)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let plan = import_plan(&files, expand)?;
    let mut report = serde_json::to_vec_pretty(&plan.report)?;
    report.push(b'\n');
    PreparedDirectory::from_plan(
        boundary,
        ExportPlan {
            files: BTreeMap::from([
                ("rebuilt.nes".into(), plan.rom),
                ("verification.json".into(), report),
            ]),
            summary: if expand {
                "文本回注、256 KiB PRG 扩容与重提取核对完成。".into()
            } else {
                "文本回注与重提取核对完成，保持原 PRG 容量。".into()
            },
        },
    )
}

pub fn prepare_chinese_import(
    workspace: &Path,
    translations: &Path,
    font_manifest: &Path,
    font_data: &Path,
    output: &Path,
) -> Result<PreparedDirectory> {
    let names = INPUT_FILES
        .iter()
        .filter_map(|name| name.strip_prefix("translations/"))
        .collect::<Vec<_>>();
    let mut inputs = vec![workspace.to_path_buf(), translations.to_path_buf()];
    inputs.extend(INPUT_FILES.iter().map(|name| workspace.join(name)));
    inputs.extend(names.iter().map(|name| translations.join(name)));
    inputs.extend([font_manifest.to_path_buf(), font_data.to_path_buf()]);
    let boundary = OutputBoundary::prepare(&inputs, output)?;
    let original = INPUT_FILES
        .iter()
        .zip(boundary.inputs.iter().skip(2))
        .map(|(name, input)| Ok(((*name).to_owned(), read_file(&input.resolved)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let translated = names
        .iter()
        .zip(boundary.inputs.iter().skip(2 + INPUT_FILES.len()))
        .map(|(name, input)| Ok(((*name).to_owned(), read_file(&input.resolved)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let count = boundary.inputs.len();
    let plan = chinese_import_plan(
        &original,
        &translated,
        &read_file(&boundary.inputs[count - 2].resolved)?,
        &read_file(&boundary.inputs[count - 1].resolved)?,
    )?;
    PreparedDirectory::from_plan(boundary, plan)
}

pub fn prepare_chinese_export(workspace: &Path, output: &Path) -> Result<PreparedDirectory> {
    let names = ["rebuilt.nes", "config.json", "font_manifest.json"];
    let mut inputs = vec![workspace.to_path_buf()];
    inputs.extend(names.iter().map(|name| workspace.join(name)));
    let boundary = OutputBoundary::prepare(&inputs, output)?;
    let files = boundary
        .inputs
        .iter()
        .skip(1)
        .map(|input| read_file(&input.resolved))
        .collect::<Result<Vec<_>>>()?;
    PreparedDirectory::from_plan(
        boundary,
        chinese_export_plan(&files[0], &files[1], &files[2])?,
    )
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    if !fs::metadata(path)?.is_file() {
        return Err(format!("输入必须是文件: {}", path.display()).into());
    }
    fs::read(path).map_err(|error| format!("无法读取输入 {}: {error}", path.display()).into())
}

#[derive(Debug)]
struct ProtectedInput {
    requested: PathBuf,
    resolved: PathBuf,
}

#[derive(Debug)]
struct OutputBoundary {
    inputs: Vec<ProtectedInput>,
    requested: PathBuf,
    output: PathBuf,
}

impl OutputBoundary {
    fn prepare(inputs: &[PathBuf], output: &Path) -> Result<Self> {
        let requested = absolute_nonempty(output)?;
        let output = resolve_nearest_existing(&requested)?;
        let inputs = inputs
            .iter()
            .map(|path| {
                Ok(ProtectedInput {
                    requested: absolute_nonempty(path)?,
                    resolved: fs::canonicalize(path)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let boundary = Self {
            inputs,
            requested,
            output,
        };
        boundary.recheck()?;
        Ok(boundary)
    }

    fn recheck(&self) -> Result<()> {
        let current = resolve_nearest_existing(&self.requested)?;
        if !same_path(&current, &self.output) {
            return Err("输出的父路径或目录联接在预检后发生变化，请重新预检".into());
        }
        if current.file_name().is_none() {
            return Err("输出必须是新的目录名，不能使用文件系统根目录".into());
        }
        for input in &self.inputs {
            let now = resolve_nearest_existing(&input.requested)?;
            if overlaps(&current, &input.resolved) || overlaps(&current, &now) {
                return Err(format!(
                    "输出与受保护输入重叠: {} / {}",
                    self.requested.display(),
                    input.requested.display()
                )
                .into());
            }
        }
        match fs::symlink_metadata(&self.requested) {
            Ok(_) => Err(format!(
                "输出已存在: {}。本工具只提交到新目录；即使 --overwrite 也不覆盖已有目录。",
                self.requested.display()
            )
            .into()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

fn absolute_nonempty(path: &Path) -> Result<PathBuf> {
    if path.as_os_str().is_empty() {
        return Err("路径不能为空".into());
    }
    Ok(std::path::absolute(path)?)
}

// Resolve the actual existing parent before appending absent components, preserving junctions.
fn resolve_nearest_existing(path: &Path) -> Result<PathBuf> {
    let mut ancestor = absolute_nonempty(path)?;
    let mut missing = Vec::new();
    let mut resolved = loop {
        match fs::canonicalize(&ancestor) {
            Ok(path) => break path,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let part = ancestor.components().next_back().ok_or("无法解析路径根")?;
                if !matches!(
                    part,
                    Component::Normal(_) | Component::ParentDir | Component::CurDir
                ) {
                    return Err(format!("路径根不存在: {}", ancestor.display()).into());
                }
                missing.push(part.as_os_str().to_os_string());
                ancestor.pop();
            }
            Err(error) => return Err(error.into()),
        }
    };
    for part in missing.into_iter().rev() {
        if part == ".." {
            if !resolved.pop() {
                return Err("路径越过文件系统根目录".into());
            }
        } else if part != "." {
            resolved.push(part);
        }
    }
    Ok(resolved)
}

fn comparison_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(path.as_os_str().to_string_lossy().to_lowercase())
    }
    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    comparison_path(left) == comparison_path(right)
}

fn overlaps(left: &Path, right: &Path) -> bool {
    let left = comparison_path(left);
    let right = comparison_path(right);
    left.starts_with(&right) || right.starts_with(&left)
}

fn normalize_plan_path(name: &str) -> Result<PathBuf> {
    if name.is_empty() || name.contains(['\0', ':']) || Path::new(name).is_absolute() {
        return Err(format!("不安全的计划文件路径: {name:?}").into());
    }
    let mut path = PathBuf::new();
    for part in name.split(['/', '\\']) {
        if part.is_empty() || part == "." || part == ".." {
            return Err(format!("计划文件路径不允许空组件、. 或 ..: {name:?}").into());
        }
        path.push(part);
    }
    if path
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("计划文件必须使用相对路径: {name:?}").into());
    }
    Ok(path)
}

fn validate_plan_paths(files: &BTreeMap<String, Vec<u8>>) -> Result<()> {
    if files.is_empty() {
        return Err("输出计划没有文件".into());
    }
    let mut paths: Vec<PathBuf> = Vec::with_capacity(files.len());
    for name in files.keys() {
        let path = comparison_path(&normalize_plan_path(name)?);
        if paths.iter().any(|prior| overlaps(prior, &path)) {
            return Err(format!("计划文件存在重复或文件/目录冲突: {name:?}").into());
        }
        paths.push(path);
    }
    Ok(())
}

struct CreatedParents {
    paths: Vec<PathBuf>,
    keep: bool,
}

impl CreatedParents {
    fn ensure(parent: &Path) -> Result<Self> {
        let mut missing = Vec::new();
        let mut ancestor = parent.to_path_buf();
        loop {
            match fs::symlink_metadata(&ancestor) {
                Ok(_) => {
                    if !fs::metadata(&ancestor)?.is_dir()
                        || !same_path(&fs::canonicalize(&ancestor)?, &ancestor)
                    {
                        return Err("输出父路径已变化或不是目录，请重新预检".into());
                    }
                    break;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    missing.push(ancestor.clone());
                    if !ancestor.pop() {
                        return Err("无法定位输出的已有父目录".into());
                    }
                }
                Err(error) => return Err(error.into()),
            }
        }
        let mut guard = Self {
            paths: Vec::new(),
            keep: false,
        };
        for path in missing.into_iter().rev() {
            match fs::create_dir(&path) {
                Ok(()) => guard.paths.push(path.clone()),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.into()),
            }
            if !fs::metadata(&path)?.is_dir() || !same_path(&fs::canonicalize(&path)?, &path) {
                return Err("创建输出父目录期间路径发生变化".into());
            }
        }
        Ok(guard)
    }
}

impl Drop for CreatedParents {
    fn drop(&mut self) {
        if !self.keep {
            for path in self.paths.iter().rev() {
                // Remove only empty directories created by this operation.
                if fs::canonicalize(path).is_ok_and(|current| same_path(&current, path)) {
                    let _ = fs::remove_dir(path);
                }
            }
        }
    }
}

struct StagingDirectory {
    path: PathBuf,
    parent: PathBuf,
}

impl StagingDirectory {
    fn create(parent: &Path) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let parent = fs::canonicalize(parent)?;
        for _ in 0..100 {
            let path = parent.join(format!(
                ".sekiryuuou-stage-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path, parent }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Err("无法分配独立的暂存目录".into())
    }
}

impl Drop for StagingDirectory {
    fn drop(&mut self) {
        if self
            .path
            .parent()
            .is_some_and(|parent| same_path(parent, &self.parent))
            && fs::canonicalize(&self.path).is_ok_and(|current| same_path(&current, &self.path))
        {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

#[cfg(windows)]
fn publish_new_directory(stage: &Path, output: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
    }
    let stage: Vec<u16> = stage.as_os_str().encode_wide().chain(Some(0)).collect();
    let output: Vec<u16> = output.as_os_str().encode_wide().chain(Some(0)).collect();
    // Without MOVEFILE_REPLACE_EXISTING the commit also rejects concurrent destinations.
    if unsafe { MoveFileExW(stage.as_ptr(), output.as_ptr(), 0) } == 0 {
        return Err(io::Error::last_os_error().into());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn publish_new_directory(stage: &Path, output: &Path) -> Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    unsafe extern "C" {
        fn renameat2(
            old_dir: i32,
            old_path: *const std::ffi::c_char,
            new_dir: i32,
            new_path: *const std::ffi::c_char,
            flags: u32,
        ) -> i32;
    }
    let stage = CString::new(stage.as_os_str().as_bytes())?;
    let output = CString::new(output.as_os_str().as_bytes())?;
    // AT_FDCWD and RENAME_NOREPLACE preserve the same no-overwrite contract as Windows.
    if unsafe { renameat2(-100, stage.as_ptr(), -100, output.as_ptr(), 1) } != 0 {
        return Err(io::Error::last_os_error().into());
    }
    Ok(())
}

#[cfg(not(any(windows, target_os = "linux")))]
fn publish_new_directory(_stage: &Path, _output: &Path) -> Result<()> {
    Err("当前平台尚未实现禁止覆盖的目录事务提交；支持 Windows 和 Linux".into())
}

struct Export;
struct Import;
struct Cycle;
struct ChineseLayout;
struct ChineseExport;

fn source_fields() -> Vec<Field> {
    vec![
        Field::new("source", "原始 NES ROM", FieldKind::Path).required(),
        Field::new("output", "新建输出目录", FieldKind::Path).required(),
    ]
}

fn prefill_path(paths: &[PathBuf], parameters: &mut Parameters, input: &str, suffix: &str) {
    if let Some(path) = paths.first() {
        if parameters.get(input).is_none() {
            parameters.set(input, Value::Path(path.clone()));
        }
        if parameters.get("output").is_none() {
            if let Some(stem) = path.file_stem() {
                let mut name = stem.to_os_string();
                name.push(suffix);
                parameters.set("output", Value::Path(path.with_file_name(name)));
            }
        }
    }
}

fn cli_error(error: impl std::fmt::Display) -> vn_cli::Error {
    vn_cli::Error(error.to_string())
}

struct DirectoryJob {
    preview: Preview,
    prepared: PreparedDirectory,
}

fn job(prepared: PreparedDirectory, steps: &[&str]) -> Box<dyn PreparedOperation> {
    Box::new(DirectoryJob {
        preview: prepared.preview(steps.iter().map(|step| (*step).to_owned()).collect()),
        prepared,
    })
}

impl PreparedOperation for DirectoryJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        progress.report("将已校验的快照写入暂存目录，并提交到新的输出目录")?;
        let report = self.prepared.commit().map_err(cli_error)?;
        Ok(RunReport {
            summary: report.summary,
            totals: vec![
                ("文件数".into(), report.file_count as u64),
                ("字节数".into(), report.byte_count),
            ],
            outputs: vec![report.output],
            ..RunReport::default()
        })
    }
}

impl Operation for Export {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new("export", "导出完整脚本与翻译 JSON", source_fields());
        spec.description = "解析 ROM，保留原生记录与来源 ROM，生成可编辑翻译 JSON。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        prefill_path(paths, parameters, "source", "-export");
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let prepared = prepare_export(parameters.path("source")?, parameters.path("output")?)
            .map_err(cli_error)?;
        Ok(job(
            prepared,
            &["解析完整原生脚本", "投影翻译 JSON 并安全写出导出目录"],
        ))
    }
}

impl Operation for Import {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "import",
            "回注文本并核对，可扩容 PRG",
            vec![
                Field::new("workspace", "导出工作区", FieldKind::Path).required(),
                Field::new("output", "新建成品目录", FieldKind::Path).required(),
                Field::new("expand", "扩容 PRG 到 256 KiB", FieldKind::Flag)
                    .default(Value::Flag(false)),
            ],
        );
        spec.description =
            "验证来源与译文，回注并重提取核对；--expand 扩容 PRG，不修改 CHR。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        prefill_path(paths, parameters, "workspace", "-rebuilt");
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let prepared = prepare_import(
            parameters.path("workspace")?,
            parameters.path("output")?,
            parameters.flag("expand"),
        )
        .map_err(cli_error)?;
        Ok(job(
            prepared,
            &[
                "验证来源、原生记录与翻译 JSON",
                "回注、按参数扩容并重提取核对",
                "安全写出 ROM 与验证报告",
            ],
        ))
    }
}

impl Operation for Cycle {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new("cycle", "验证导出、回注与扩容闭环", source_fields());
        spec.primary = false;
        spec.description =
            "对未改译文执行导出→回注→256 KiB PRG 扩容→重提取，保存完整核对结果。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        prefill_path(paths, parameters, "source", "-cycle");
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let prepared = prepare_cycle(parameters.path("source")?, parameters.path("output")?)
            .map_err(cli_error)?;
        Ok(job(
            prepared,
            &[
                "导出完整脚本",
                "未改译文回注、PRG 扩容并重提取核对",
                "安全提交闭环产物目录",
            ],
        ))
    }
}

impl Operation for ChineseLayout {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "import-zh",
            "回注完整中文并生成字库运行时",
            vec![
                Field::new("workspace", "原版导出工作区", FieldKind::Path).required(),
                Field::new("translations", "六份中文译文目录", FieldKind::Path).required(),
                Field::new("font-map", "全角字形JSON", FieldKind::Path).required(),
                Field::new("font-data", "16×16单色字模数据", FieldKind::Path).required(),
                Field::new("output", "新建中文成品目录", FieldKind::Path).required(),
            ],
        );
        spec.primary = false;
        spec.description =
            "核对原版来源与译稿，编译局部字典，生成512 KiB MMC3中文字库镜像，并接入绘制、缓存和分页程序。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let prepared = prepare_chinese_import(
            parameters.path("workspace")?,
            parameters.path("translations")?,
            parameters.path("font-map")?,
            parameters.path("font-data")?,
            parameters.path("output")?,
        )
        .map_err(cli_error)?;
        Ok(job(
            prepared,
            &[
                "核对译稿和字库",
                "回注中文、接入字库与分页程序并重提取核对",
                "安全保存新镜像、原始快照与运行时元数据",
            ],
        ))
    }
}

impl Operation for ChineseExport {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "export-zh",
            "从中文镜像重提取译文",
            vec![
                Field::new("workspace", "中文成品目录", FieldKind::Path).required(),
                Field::new("output", "新建译文目录", FieldKind::Path).required(),
            ],
        );
        spec.primary = false;
        spec.description = "绑定rebuilt.nes/config.json/font_manifest.json，重新解码镜像字节和局部字典，生成六份中文译文。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let prepared =
            prepare_chinese_export(parameters.path("workspace")?, parameters.path("output")?)
                .map_err(cli_error)?;
        Ok(job(
            prepared,
            &["核对中文镜像身份和记录定位", "从镜像重新解码中文并安全保存"],
        ))
    }
}

pub fn run() -> vn_cli::Result<()> {
    Panel::new(
        std::env::current_exe()?.into_os_string(),
        "赤龙王脚本导出与回注",
        env!("CARGO_PKG_VERSION"),
        vec![
            Box::new(Export),
            Box::new(Import),
            Box::new(Cycle),
            Box::new(ChineseLayout),
            Box::new(ChineseExport),
        ],
    )?
    .run_env()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> StagingDirectory {
        StagingDirectory::create(&std::env::temp_dir()).expect("创建测试目录")
    }

    fn prepared(
        source: &Path,
        output: &Path,
        files: BTreeMap<String, Vec<u8>>,
    ) -> PreparedDirectory {
        PreparedDirectory::from_plan(
            OutputBoundary::prepare(&[source.to_path_buf()], output).unwrap(),
            ExportPlan {
                files,
                summary: "测试快照".into(),
            },
        )
        .unwrap()
    }

    #[test]
    fn rejects_input_overlap_and_existing_output() {
        let temp = scratch();
        let source = temp.path.join("source.nes");
        fs::write(&source, b"source").unwrap();
        assert!(OutputBoundary::prepare(std::slice::from_ref(&source), &source).is_err());
        assert!(OutputBoundary::prepare(std::slice::from_ref(&source), &temp.path).is_err());
        let workspace = temp.path.join("workspace");
        fs::create_dir(&workspace).unwrap();
        assert!(OutputBoundary::prepare(
            std::slice::from_ref(&workspace),
            &workspace.join("new-output")
        )
        .is_err());
        let existing = temp.path.join("existing");
        fs::create_dir(&existing).unwrap();
        assert!(OutputBoundary::prepare(&[source], &existing).is_err());
    }

    #[test]
    fn rejects_unsafe_plan_paths_and_file_directory_collisions() {
        for name in [
            "../escape",
            "/absolute",
            "C:\\absolute",
            "a/./b",
            "a//b",
            "a\\..\\b",
        ] {
            assert!(validate_plan_paths(&BTreeMap::from([(name.into(), Vec::new())])).is_err());
        }
        assert!(validate_plan_paths(&BTreeMap::from([
            ("a".into(), Vec::new()),
            ("a/b".into(), Vec::new()),
        ]))
        .is_err());
        assert!(validate_plan_paths(&BTreeMap::from([
            ("a/b".into(), Vec::new()),
            ("a\\b".into(), Vec::new()),
        ]))
        .is_err());
    }

    #[test]
    fn rejects_destination_created_after_preparation() {
        let temp = scratch();
        let source = temp.path.join("source.nes");
        fs::write(&source, b"source").unwrap();
        let output = temp.path.join("result");
        let plan = prepared(
            &source,
            &output,
            BTreeMap::from([("data.bin".into(), vec![1])]),
        );
        fs::create_dir(&output).unwrap();
        fs::write(output.join("keep.txt"), b"keep").unwrap();
        assert!(plan.commit().is_err());
        assert_eq!(fs::read(output.join("keep.txt")).unwrap(), b"keep");
    }

    #[test]
    fn publication_does_not_replace_even_an_empty_destination() {
        let temp = scratch();
        let stage = StagingDirectory::create(&temp.path).unwrap();
        fs::write(stage.path.join("data.bin"), [1, 2, 3]).unwrap();
        let output = temp.path.join("existing");
        fs::create_dir(&output).unwrap();
        assert!(publish_new_directory(&stage.path, &output).is_err());
        assert!(output.is_dir());
        assert!(!output.join("data.bin").exists());
        assert!(stage.path.join("data.bin").exists());
    }

    #[test]
    fn failed_prepare_is_read_only() {
        let temp = scratch();
        let source = temp.path.join("invalid.nes");
        fs::write(&source, b"invalid ROM").unwrap();
        let parent = temp.path.join("new-parent");
        assert!(prepare_export(&source, &parent.join("result")).is_err());
        assert!(!parent.exists());
        assert_eq!(fs::read(source).unwrap(), b"invalid ROM");
    }

    #[test]
    fn failed_write_cleans_stage_and_new_empty_parents() {
        let temp = scratch();
        let source = temp.path.join("source.nes");
        fs::write(&source, b"source").unwrap();
        let parent = temp.path.join("new-parent");
        let output = parent.join("result");
        let plan = prepared(
            &source,
            &output,
            BTreeMap::from([("a.bin".into(), vec![1]), ("z".repeat(10_000), vec![2])]),
        );
        assert!(plan.commit().is_err());
        assert!(!output.exists());
        assert!(!parent.exists());
    }

    #[test]
    fn success_publishes_prepared_snapshot_into_new_directory() {
        let temp = scratch();
        let source = temp.path.join("source.nes");
        fs::write(&source, b"source").unwrap();
        let output = temp.path.join("new-parent").join("result");
        let plan = prepared(
            &source,
            &output,
            BTreeMap::from([("translations/data.bin".into(), vec![1, 2, 3])]),
        );
        fs::write(&source, b"later source change").unwrap();
        let report = plan.commit().unwrap();
        assert_eq!(report.file_count, 1);
        assert_eq!(report.byte_count, 3);
        assert_eq!(
            fs::read(output.join("translations/data.bin")).unwrap(),
            [1, 2, 3]
        );
        assert_eq!(fs::read(source).unwrap(), b"later source change");
    }

    #[cfg(windows)]
    fn junction(link: &Path, target: &Path) {
        let result = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "无法创建测试目录联接: {:?}",
            result
        );
    }

    #[cfg(windows)]
    #[test]
    fn junction_aliases_cannot_overlap_inputs_or_change_after_preparation() {
        let temp = scratch();
        let workspace = temp.path.join("workspace");
        fs::create_dir(&workspace).unwrap();
        let alias = temp.path.join("alias");
        junction(&alias, &workspace);
        assert!(
            OutputBoundary::prepare(std::slice::from_ref(&workspace), &alias.join("output"))
                .is_err()
        );
        let source = temp.path.join("source.nes");
        fs::write(&source, b"source").unwrap();
        let plan = prepared(
            &source,
            &alias.join("output"),
            BTreeMap::from([("data.bin".into(), vec![1])]),
        );
        fs::remove_dir(&alias).unwrap();
        let other = temp.path.join("other");
        fs::create_dir(&other).unwrap();
        junction(&alias, &other);
        assert!(plan.commit().is_err());
        assert!(!workspace.join("output").exists());
        assert!(!other.join("output").exists());
    }
}
