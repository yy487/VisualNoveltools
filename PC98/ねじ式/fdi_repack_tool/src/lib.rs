use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Component, Path, PathBuf};

pub const WORKSPACE_FORMAT: &str = "pc98-fdi-fat12-unpack-workspace-v1";

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FdiHeader {
    pub unknown_0x00: u32,
    pub disk_type: u32,
    pub data_offset: u32,
    pub data_size: u32,
    pub sector_size: u32,
    pub sectors_per_track: u32,
    pub heads: u32,
    pub cylinders: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fat12Info {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fat_copies: u8,
    pub root_entries: u16,
    pub total_sectors: u32,
    pub media_descriptor: u8,
    pub sectors_per_fat: u16,
    pub sectors_per_track: u16,
    pub heads: u16,
    pub root_directory_sectors: u32,
    pub first_data_sector: u32,
    pub data_clusters: u32,
    pub fat_copies_identical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectoryManifest {
    pub path: String,
    pub raw_short_name_hex: String,
    pub attributes: u8,
    pub directory_entry_offset: u64,
    pub start_cluster: u16,
    pub cluster_chain: Vec<u16>,
    pub dos_time: u16,
    pub dos_date: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileManifest {
    pub path: String,
    pub raw_short_name_hex: String,
    pub attributes: u8,
    pub directory_entry_offset: u64,
    pub start_cluster: u16,
    pub size: u32,
    pub cluster_chain: Vec<u16>,
    pub dos_time: u16,
    pub dos_date: u16,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveManifest {
    pub source_file: String,
    pub source_sha256: String,
    pub output_dir: String,
    pub archive_bytes: u64,
    pub fdi: FdiHeader,
    pub fat12: Fat12Info,
    pub volume_labels: Vec<String>,
    pub directories: Vec<DirectoryManifest>,
    pub files: Vec<FileManifest>,
    pub skipped_deleted_entries: u64,
    pub skipped_lfn_entries: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RolePaths {
    pub unpacked_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceManifest {
    pub _format: String,
    pub tool_version: String,
    pub role_paths: RolePaths,
    pub archives: Vec<ArchiveManifest>,
}

#[derive(Debug, Clone)]
pub struct PackReport {
    pub packed_files: usize,
    pub modified_files: usize,
    pub reallocated_files: usize,
    pub output_bytes: u64,
    pub free_clusters: usize,
    pub source_archive: String,
    pub output: PathBuf,
    pub fat_copies_identical: bool,
    pub fat_mismatch_preserved: bool,
}

#[derive(Debug, Clone)]
pub struct BatchPackMapping {
    pub source: PathBuf,
    pub workspace: PathBuf,
    pub files_root: PathBuf,
    pub output: PathBuf,
    pub archive_source_file: String,
    pub packed_files: usize,
}

#[derive(Debug, Clone)]
pub struct BatchPackPlan {
    pub mappings: Vec<BatchPackMapping>,
    pub skipped_source_images: usize,
}

#[derive(Debug, Clone)]
pub struct BatchPackReport {
    pub images: usize,
    pub packed_files: usize,
    pub modified_files: usize,
    pub reallocated_files: usize,
    pub output_bytes: u64,
    pub skipped_source_images: usize,
    pub outputs: Vec<PackReport>,
}

#[derive(Debug, Clone)]
struct DiskLayout {
    fdi: FdiHeader,
    fat12: Fat12Info,
    data_offset: usize,
    fat_offset: usize,
    fat_bytes: usize,
    cluster_bytes: usize,
    max_cluster: u16,
}

#[derive(Debug)]
struct PreparedFile {
    data: Vec<u8>,
    modified: bool,
    desired_chain: Vec<u16>,
}

#[derive(Debug)]
struct BatchJob {
    mapping: BatchPackMapping,
}

#[derive(Debug, Clone)]
struct SourceImage {
    path: PathBuf,
    sha256: String,
    bytes: u64,
}

#[derive(Debug, Clone)]
struct ArchiveCandidate {
    workspace: PathBuf,
    files_root: PathBuf,
    output_relative: PathBuf,
    source_file: String,
    source_sha256: String,
    archive_bytes: u64,
    packed_files: usize,
}

#[derive(Debug)]
struct BuiltImage {
    bytes: Vec<u8>,
    report: PackReport,
}

#[derive(Debug)]
struct PreparedBatch {
    plan: BatchPackPlan,
    images: Vec<BuiltImage>,
}

#[derive(Debug)]
struct StagedOutput {
    temporary: PathBuf,
    output: PathBuf,
    backup: Option<PathBuf>,
    committed: bool,
}

struct DirEntryExpectation<'a> {
    raw_name_hex: &'a str,
    attributes: u8,
    start_cluster: u16,
    size: Option<u32>,
    dos_time: u16,
    dos_date: u16,
    path: &'a str,
}

pub fn is_workspace_manifest(path: &Path) -> bool {
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    serde_json::from_slice::<WorkspaceManifest>(&bytes)
        .map(|manifest| manifest._format == WORKSPACE_FORMAT)
        .unwrap_or(false)
}

pub fn is_supported_fdi(path: &Path) -> bool {
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    parse_layout(path, &bytes).is_ok()
}

pub fn directory_contains_workspace(path: &Path) -> bool {
    path.is_dir()
        && find_workspace_manifests(path)
            .map(|items| !items.is_empty())
            .unwrap_or(false)
}

pub fn directory_contains_fdi(path: &Path) -> bool {
    path.is_dir()
        && collect_source_images(path)
            .map(|items| !items.is_empty())
            .unwrap_or(false)
}

pub fn pack_image(
    source_fdi: &Path,
    workspace_json: &Path,
    files_root: &Path,
    output_fdi: &Path,
    overwrite: bool,
) -> Result<PackReport> {
    validate_output_path(output_fdi, overwrite)?;
    validate_disjoint_output(
        output_fdi,
        &[
            ("源 FDI", source_fdi),
            ("工作区清单", workspace_json),
            ("成员目录", files_root),
        ],
    )?;

    let built = build_image(source_fdi, workspace_json, files_root, output_fdi)?;
    write_output(output_fdi, &built.bytes, overwrite)?;
    Ok(built.report)
}

fn build_image(
    source_fdi: &Path,
    workspace_json: &Path,
    files_root: &Path,
    output_fdi: &Path,
) -> Result<BuiltImage> {
    let source = fs::read(source_fdi)
        .map_err(|e| format!("无法读取源 FDI {}: {e}", source_fdi.display()))?;
    let layout = parse_layout(source_fdi, &source)?;
    let workspace = read_workspace(workspace_json)?;
    let source_hash = sha256_hex(&source);
    let archive = select_archive(&workspace, &source_hash, source.len())?;
    validate_manifest_archive(archive, &layout, &source)?;
    validate_files_root(files_root, archive)?;

    let mut prepared = prepare_files(files_root, archive)?;
    let modified_files = prepared.iter().filter(|item| item.modified).count();
    if modified_files == 0 {
        let report = PackReport {
            packed_files: archive.files.len(),
            modified_files: 0,
            reallocated_files: 0,
            output_bytes: source.len() as u64,
            free_clusters: count_free_clusters(first_fat(&source, &layout), layout.max_cluster)?,
            source_archive: archive.source_file.clone(),
            output: output_fdi.to_path_buf(),
            fat_copies_identical: layout.fat12.fat_copies_identical,
            fat_mismatch_preserved: !layout.fat12.fat_copies_identical,
        };
        return Ok(BuiltImage {
            bytes: source,
            report,
        });
    }

    allocate_chains(&source, &layout, archive, &mut prepared)?;
    let reallocated_files = archive
        .files
        .iter()
        .zip(&prepared)
        .filter(|(manifest, file)| manifest.cluster_chain != file.desired_chain)
        .count();
    let mut rebuilt = source.clone();
    rebuild_fat(&mut rebuilt, &layout, archive, &prepared)?;
    rebuild_files(&mut rebuilt, &layout, archive, &prepared)?;
    validate_rebuilt(&rebuilt, &source, &layout, archive, &prepared)?;
    let free_clusters = count_free_clusters(first_fat(&rebuilt, &layout), layout.max_cluster)?;
    let report = PackReport {
        packed_files: archive.files.len(),
        modified_files,
        reallocated_files,
        output_bytes: rebuilt.len() as u64,
        free_clusters,
        source_archive: archive.source_file.clone(),
        output: output_fdi.to_path_buf(),
        fat_copies_identical: layout.fat12.fat_copies_identical,
        fat_mismatch_preserved: !layout.fat12.fat_copies_identical,
    };
    Ok(BuiltImage {
        bytes: rebuilt,
        report,
    })
}

pub fn preview_workspace_pack(
    source_input: &Path,
    unpacked_root: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<BatchPackPlan> {
    Ok(prepare_workspace_pack(source_input, unpacked_root, output, overwrite)?.plan)
}

pub fn pack_workspace_tree(
    source_input: &Path,
    unpacked_root: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<BatchPackReport> {
    let prepared = prepare_workspace_pack(source_input, unpacked_root, output, overwrite)?;
    let output_is_directory = source_input.is_dir();
    let output_root_existed = output_is_directory && output.exists();
    if output_is_directory && !output_root_existed {
        fs::create_dir(output)
            .map_err(|e| format!("无法创建输出根目录 {}: {e}", output.display()))?;
    }

    let write_result = (|| -> Result<()> {
        for image in &prepared.images {
            let parent =
                image.report.output.parent().ok_or_else(|| {
                    format!("输出路径没有父目录: {}", image.report.output.display())
                })?;
            fs::create_dir_all(parent)
                .map_err(|e| format!("无法创建输出目录 {}: {e}", parent.display()))?;
        }
        let payloads: Vec<_> = prepared
            .images
            .iter()
            .map(|image| (image.report.output.as_path(), image.bytes.as_slice()))
            .collect();
        write_outputs_transactional(&payloads, overwrite)?;
        Ok(())
    })();
    if write_result.is_err() && output_is_directory && !output_root_existed {
        let _ = fs::remove_dir_all(output);
    }
    write_result?;

    let outputs: Vec<_> = prepared
        .images
        .into_iter()
        .map(|image| image.report)
        .collect();
    Ok(BatchPackReport {
        images: outputs.len(),
        packed_files: outputs.iter().map(|item| item.packed_files).sum(),
        modified_files: outputs.iter().map(|item| item.modified_files).sum(),
        reallocated_files: outputs.iter().map(|item| item.reallocated_files).sum(),
        output_bytes: outputs.iter().map(|item| item.output_bytes).sum(),
        skipped_source_images: prepared.plan.skipped_source_images,
        outputs,
    })
}

fn prepare_workspace_pack(
    source_input: &Path,
    unpacked_root: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<PreparedBatch> {
    let jobs = discover_batch_jobs(source_input, unpacked_root, output, overwrite)?;
    let skipped_source_images = count_skipped_source_images(source_input, &jobs)?;
    let mut images = Vec::with_capacity(jobs.len());
    for job in &jobs {
        images.push(build_image(
            &job.mapping.source,
            &job.mapping.workspace,
            &job.mapping.files_root,
            &job.mapping.output,
        )?);
    }
    Ok(PreparedBatch {
        plan: BatchPackPlan {
            mappings: jobs.into_iter().map(|job| job.mapping).collect(),
            skipped_source_images,
        },
        images,
    })
}

fn discover_batch_jobs(
    source_input: &Path,
    unpacked_root: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<Vec<BatchJob>> {
    validate_plain_directory(unpacked_root, "解包根目录")?;
    let workspace_paths = find_workspace_manifests(unpacked_root)?;
    if workspace_paths.is_empty() {
        return Err(format!(
            "解包根目录中未递归找到有效工作区清单: {}",
            unpacked_root.display()
        ));
    }
    let archives = collect_archive_candidates(unpacked_root, &workspace_paths)?;
    let source_images = collect_source_images(source_input)?;
    if source_images.is_empty() {
        return Err(format!(
            "原始镜像输入中未识别出有效 FDI: {}",
            source_input.display()
        ));
    }

    let source_is_directory = source_input.is_dir();
    validate_batch_output(
        source_input,
        unpacked_root,
        output,
        source_is_directory,
        overwrite,
    )?;
    let selected_archives: Vec<&ArchiveCandidate> = if source_is_directory {
        archives.iter().collect()
    } else {
        let source = source_images
            .first()
            .ok_or_else(|| "原始 FDI 输入为空".to_string())?;
        let matches: Vec<_> = archives
            .iter()
            .filter(|archive| source_matches_archive(source, archive))
            .collect();
        match matches.as_slice() {
            [archive] => vec![*archive],
            [] => {
                return Err(format!(
                    "原始 FDI SHA-256 {} 未在递归发现的工作区中找到记录",
                    source.sha256
                ));
            }
            _ => {
                return Err(format!(
                    "原始 FDI SHA-256 {} 在多个工作区记录中匹配，无法确定成员树",
                    source.sha256
                ));
            }
        }
    };

    let mut output_keys = HashSet::new();
    let mut jobs = Vec::with_capacity(selected_archives.len());
    for archive in selected_archives {
        let matches: Vec<_> = source_images
            .iter()
            .filter(|source| source_matches_archive(source, archive))
            .collect();
        let source = match matches.as_slice() {
            [source] => *source,
            [] => {
                return Err(format!(
                    "缺少清单记录所需的原始 FDI: {} (SHA-256 {})",
                    archive.source_file, archive.source_sha256
                ));
            }
            _ => {
                let paths: Vec<_> = matches
                    .iter()
                    .map(|item| item.path.display().to_string())
                    .collect();
                return Err(format!(
                    "多个原始 FDI 与清单记录 {} 使用相同 SHA-256，无法唯一选择: {paths:?}",
                    archive.source_file
                ));
            }
        };
        let target = if source_is_directory {
            output.join(&archive.output_relative)
        } else {
            output.to_path_buf()
        };
        validate_batch_target(&target, overwrite)?;
        validate_batch_target_ancestor(&target)?;
        let target_key = target.to_string_lossy().to_lowercase();
        if !output_keys.insert(target_key) {
            return Err(format!(
                "多个工作区记录映射到同一输出: {}",
                target.display()
            ));
        }
        jobs.push(BatchJob {
            mapping: BatchPackMapping {
                source: source.path.clone(),
                workspace: archive.workspace.clone(),
                files_root: archive.files_root.clone(),
                output: target,
                archive_source_file: archive.source_file.clone(),
                packed_files: archive.packed_files,
            },
        });
    }
    jobs.sort_by(|left, right| {
        left.mapping
            .output
            .to_string_lossy()
            .cmp(&right.mapping.output.to_string_lossy())
    });
    Ok(jobs)
}

fn count_skipped_source_images(source_input: &Path, jobs: &[BatchJob]) -> Result<usize> {
    if source_input.is_file() {
        return Ok(0);
    }
    let sources = collect_source_images(source_input)?;
    let used: HashSet<_> = jobs
        .iter()
        .map(|job| job.mapping.source.to_string_lossy().to_lowercase())
        .collect();
    Ok(sources
        .iter()
        .filter(|source| !used.contains(&source.path.to_string_lossy().to_lowercase()))
        .count())
}

fn source_matches_archive(source: &SourceImage, archive: &ArchiveCandidate) -> bool {
    source.bytes == archive.archive_bytes
        && source.sha256.eq_ignore_ascii_case(&archive.source_sha256)
}

fn find_workspace_manifests(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_plain_files(root, root, 0, &mut files, "解包根目录")?;
    let mut workspaces: Vec<_> = files
        .into_iter()
        .filter(|path| is_workspace_manifest(path))
        .collect();
    workspaces.sort_by_key(|path| path.to_string_lossy().to_lowercase());
    Ok(workspaces)
}

fn collect_archive_candidates(
    unpacked_root: &Path,
    workspace_paths: &[PathBuf],
) -> Result<Vec<ArchiveCandidate>> {
    let mut candidates = Vec::new();
    for workspace_path in workspace_paths {
        let workspace = read_workspace(workspace_path)?;
        let workspace_parent = workspace_path
            .parent()
            .ok_or_else(|| format!("工作区清单没有父目录: {}", workspace_path.display()))?;
        let workspace_relative = workspace_parent
            .strip_prefix(unpacked_root)
            .map_err(|_| format!("工作区清单不在解包根目录内: {}", workspace_path.display()))?;
        let managed_root = if workspace.role_paths.unpacked_root == "." {
            workspace_parent.to_path_buf()
        } else {
            join_member_path(workspace_parent, &workspace.role_paths.unpacked_root)?
        };
        for archive in workspace.archives {
            validate_relative_role_path(&archive.source_file, false)?;
            validate_relative_role_path(&archive.output_dir, false)?;
            validate_hash(&archive.source_sha256, "source_sha256")?;
            let files_root = join_member_path(&managed_root, &archive.output_dir)?;
            let mut output_relative = workspace_relative.to_path_buf();
            output_relative = join_member_path(&output_relative, &archive.source_file)?;
            candidates.push(ArchiveCandidate {
                workspace: workspace_path.clone(),
                files_root,
                output_relative,
                source_file: archive.source_file,
                source_sha256: archive.source_sha256,
                archive_bytes: archive.archive_bytes,
                packed_files: archive.files.len(),
            });
        }
    }
    if candidates.is_empty() {
        return Err("递归发现的工作区均不含 archives".to_string());
    }
    Ok(candidates)
}

fn collect_source_images(input: &Path) -> Result<Vec<SourceImage>> {
    let metadata = fs::symlink_metadata(input)
        .map_err(|e| format!("无法读取原始镜像输入 {}: {e}", input.display()))?;
    if is_reparse_point(&metadata) || metadata.file_type().is_symlink() {
        return Err(format!(
            "原始镜像输入不能是重解析点或符号链接: {}",
            input.display()
        ));
    }
    let paths = if metadata.is_file() {
        vec![input.to_path_buf()]
    } else if metadata.is_dir() {
        let mut paths = Vec::new();
        collect_plain_files(input, input, 0, &mut paths, "原始镜像目录")?;
        paths.sort_by_key(|path| path.to_string_lossy().to_lowercase());
        paths
    } else {
        return Err(format!(
            "原始镜像输入必须是普通文件或目录: {}",
            input.display()
        ));
    };

    let explicit_file = metadata.is_file();
    let mut images = Vec::new();
    for path in paths {
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if explicit_file => {
                return Err(format!("无法读取原始镜像候选 {}: {error}", path.display()));
            }
            Err(_) => continue,
        };
        match parse_layout(&path, &bytes) {
            Ok(_) => images.push(SourceImage {
                path,
                sha256: sha256_hex(&bytes),
                bytes: bytes.len() as u64,
            }),
            Err(error) if explicit_file => return Err(error),
            Err(_) => {}
        }
    }
    Ok(images)
}

fn collect_plain_files(
    root: &Path,
    current: &Path,
    depth: usize,
    files: &mut Vec<PathBuf>,
    role: &str,
) -> Result<()> {
    if depth > 64 {
        return Err(format!("{role}嵌套超过 64 层: {}", current.display()));
    }
    let mut entries = Vec::new();
    for entry in
        fs::read_dir(current).map_err(|e| format!("无法枚举{role} {}: {e}", current.display()))?
    {
        entries.push(entry.map_err(|e| format!("无法读取{role}目录项: {e}"))?);
    }
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_lowercase());
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|e| format!("无法读取{role}元数据 {}: {e}", path.display()))?;
        if is_reparse_point(&metadata) || metadata.file_type().is_symlink() {
            return Err(format!(
                "{role}不允许重解析点或符号链接: {}",
                path.display()
            ));
        }
        if metadata.is_dir() {
            collect_plain_files(root, &path, depth + 1, files, role)?;
        } else if metadata.is_file() {
            files.push(path);
        } else {
            return Err(format!("{role}含非普通文件: {}", path.display()));
        }
    }
    let _ = root;
    Ok(())
}

fn validate_plain_directory(path: &Path, role: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|e| format!("无法读取{role} {}: {e}", path.display()))?;
    if is_reparse_point(&metadata) || metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{role}必须是普通目录: {}", path.display()));
    }
    Ok(())
}

fn validate_batch_output(
    source_input: &Path,
    unpacked_root: &Path,
    output: &Path,
    output_is_directory: bool,
    overwrite: bool,
) -> Result<()> {
    if output_is_directory {
        if output.exists() {
            let metadata = fs::symlink_metadata(output)
                .map_err(|e| format!("无法读取输出根目录 {}: {e}", output.display()))?;
            if is_reparse_point(&metadata)
                || metadata.file_type().is_symlink()
                || !metadata.is_dir()
            {
                return Err(format!("批量输出必须是普通目录: {}", output.display()));
            }
        } else {
            let parent = output
                .parent()
                .ok_or_else(|| format!("输出根目录没有父目录: {}", output.display()))?;
            validate_plain_directory(parent, "输出根目录的父目录")?;
        }
    } else {
        validate_output_path(output, overwrite)?;
    }
    validate_disjoint_output(
        output,
        &[
            ("原始镜像输入", source_input),
            ("解包根目录", unpacked_root),
        ],
    )
}

fn validate_batch_target(target: &Path, overwrite: bool) -> Result<()> {
    if !target.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(target)
        .map_err(|e| format!("无法读取批量输出目标 {}: {e}", target.display()))?;
    if is_reparse_point(&metadata) || metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("批量输出目标必须是普通文件: {}", target.display()));
    }
    if !overwrite {
        return Err(format!(
            "输出已存在；请修改路径或显式指定 --overwrite: {}",
            target.display()
        ));
    }
    Ok(())
}

fn validate_batch_target_ancestor(target: &Path) -> Result<()> {
    let mut ancestor = target
        .parent()
        .ok_or_else(|| format!("批量输出目标没有父目录: {}", target.display()))?;
    while !ancestor.exists() {
        ancestor = ancestor
            .parent()
            .ok_or_else(|| format!("批量输出目标没有现存祖先目录: {}", target.display()))?;
    }
    validate_plain_directory(ancestor, "批量输出目标的祖先目录")
}

fn read_workspace(path: &Path) -> Result<WorkspaceManifest> {
    let bytes =
        fs::read(path).map_err(|e| format!("无法读取工作区清单 {}: {e}", path.display()))?;
    let manifest: WorkspaceManifest = serde_json::from_slice(&bytes)
        .map_err(|e| format!("工作区清单不是有效 UTF-8 JSON {}: {e}", path.display()))?;
    if manifest._format != WORKSPACE_FORMAT {
        return Err(format!(
            "工作区格式不受支持: {:?}，需要 {WORKSPACE_FORMAT}",
            manifest._format
        ));
    }
    validate_relative_role_path(&manifest.role_paths.unpacked_root, true)?;
    if manifest.archives.is_empty() {
        return Err("工作区清单不含 archives".to_string());
    }
    Ok(manifest)
}

fn select_archive<'a>(
    workspace: &'a WorkspaceManifest,
    source_hash: &str,
    source_len: usize,
) -> Result<&'a ArchiveManifest> {
    let matches: Vec<_> = workspace
        .archives
        .iter()
        .filter(|archive| {
            archive.source_sha256.eq_ignore_ascii_case(source_hash)
                && archive.archive_bytes == source_len as u64
        })
        .collect();
    match matches.as_slice() {
        [archive] => Ok(*archive),
        [] => Err(format!(
            "源 FDI SHA-256 {source_hash} 未在工作区清单中找到唯一记录"
        )),
        _ => Err(format!(
            "工作区清单含多个相同源 FDI SHA-256 记录: {source_hash}"
        )),
    }
}

fn validate_manifest_archive(
    archive: &ArchiveManifest,
    layout: &DiskLayout,
    source: &[u8],
) -> Result<()> {
    validate_hash(&archive.source_sha256, "source_sha256")?;
    validate_relative_role_path(&archive.output_dir, false)?;
    if archive.archive_bytes != source.len() as u64 {
        return Err("清单 archive_bytes 与源 FDI 长度不一致".to_string());
    }
    if archive.fdi != layout.fdi {
        return Err("清单 FDI 头字段与源盘不一致".to_string());
    }
    if archive.fat12 != layout.fat12 {
        return Err("清单 FAT12 参数与源盘不一致".to_string());
    }
    let fat = first_fat(source, layout);
    let mut paths = HashSet::new();
    let mut offsets = HashSet::new();
    let mut owners = HashSet::new();
    for directory in &archive.directories {
        validate_member_path(&directory.path)?;
        insert_unique(&mut paths, &directory.path)?;
        insert_offset(
            &mut offsets,
            directory.directory_entry_offset,
            &directory.path,
        )?;
        if directory.attributes & 0x10 == 0 {
            return Err(format!("目录 {} 未设置目录属性", directory.path));
        }
        validate_directory_entry(
            source,
            directory.directory_entry_offset,
            &DirEntryExpectation {
                raw_name_hex: &directory.raw_short_name_hex,
                attributes: directory.attributes,
                start_cluster: directory.start_cluster,
                size: None,
                dos_time: directory.dos_time,
                dos_date: directory.dos_date,
                path: &directory.path,
            },
        )?;
        validate_chain(
            fat,
            layout.max_cluster,
            directory.start_cluster,
            &directory.cluster_chain,
            &directory.path,
            &mut owners,
        )?;
    }
    for file in &archive.files {
        validate_member_path(&file.path)?;
        insert_unique(&mut paths, &file.path)?;
        insert_offset(&mut offsets, file.directory_entry_offset, &file.path)?;
        validate_hash(&file.sha256, &format!("{} sha256", file.path))?;
        if file.attributes & 0x10 != 0 || file.attributes & 0x08 != 0 {
            return Err(format!("文件 {} 的属性被标记为目录或卷标", file.path));
        }
        validate_directory_entry(
            source,
            file.directory_entry_offset,
            &DirEntryExpectation {
                raw_name_hex: &file.raw_short_name_hex,
                attributes: file.attributes,
                start_cluster: file.start_cluster,
                size: Some(file.size),
                dos_time: file.dos_time,
                dos_date: file.dos_date,
                path: &file.path,
            },
        )?;
        let needed = clusters_for_size(file.size as u64, layout.cluster_bytes)?;
        if needed != file.cluster_chain.len() {
            return Err(format!(
                "清单文件 {} 大小需要 {needed} 簇，但记录了 {} 簇",
                file.path,
                file.cluster_chain.len()
            ));
        }
        if file.size == 0 {
            if file.start_cluster != 0 || !file.cluster_chain.is_empty() {
                return Err(format!("空文件 {} 的起始簇或簇链非空", file.path));
            }
        } else {
            validate_chain(
                fat,
                layout.max_cluster,
                file.start_cluster,
                &file.cluster_chain,
                &file.path,
                &mut owners,
            )?;
        }
        let source_member = read_file_from_chain(
            source,
            layout,
            &file.cluster_chain,
            usize::try_from(file.size).map_err(|_| format!("文件 {} 过大", file.path))?,
        )?;
        if !sha256_hex(&source_member).eq_ignore_ascii_case(&file.sha256) {
            return Err(format!("源盘成员 {} 的 SHA-256 与清单不一致", file.path));
        }
    }
    validate_manifest_directory_topology(archive, layout, source, &offsets)?;
    Ok(())
}

fn validate_manifest_directory_topology(
    archive: &ArchiveManifest,
    layout: &DiskLayout,
    source: &[u8],
    manifest_offsets: &HashSet<u64>,
) -> Result<()> {
    let root_slots = root_directory_slots(layout)?;
    let mut regions = HashMap::new();
    regions.insert(String::new(), root_slots.clone());
    for directory in &archive.directories {
        regions.insert(
            directory.path.clone(),
            directory_chain_slots(layout, &directory.cluster_chain)?,
        );
    }

    for (path, offset) in archive
        .directories
        .iter()
        .map(|entry| (&entry.path, entry.directory_entry_offset))
        .chain(
            archive
                .files
                .iter()
                .map(|entry| (&entry.path, entry.directory_entry_offset)),
        )
    {
        let parent = path
            .rsplit_once('/')
            .map(|(parent, _)| parent)
            .unwrap_or("");
        let slots = regions
            .get(parent)
            .ok_or_else(|| format!("清单成员 {path} 的父目录 {parent:?} 不存在"))?;
        if !slots.contains(&offset) {
            return Err(format!(
                "清单成员 {path} 的目录项偏移 0x{offset:X} 不属于其父目录"
            ));
        }
    }

    let mut active_offsets = active_directory_offsets(source, &root_slots)?;
    for directory in &archive.directories {
        let slots = regions
            .get(&directory.path)
            .expect("directory region was inserted");
        active_offsets.extend(active_directory_offsets(source, slots)?);
    }
    if active_offsets != *manifest_offsets {
        let missing: Vec<_> = active_offsets
            .difference(manifest_offsets)
            .copied()
            .collect();
        let invalid: Vec<_> = manifest_offsets
            .difference(&active_offsets)
            .copied()
            .collect();
        return Err(format!(
            "清单目录项与源盘活动目录项不完整对应：清单缺少偏移={missing:X?}，无效偏移={invalid:X?}"
        ));
    }
    Ok(())
}

fn root_directory_slots(layout: &DiskLayout) -> Result<Vec<u64>> {
    let bps = usize::from(layout.fat12.bytes_per_sector);
    let first_root_sector = usize::from(layout.fat12.reserved_sectors)
        .checked_add(
            usize::from(layout.fat12.fat_copies)
                .checked_mul(usize::from(layout.fat12.sectors_per_fat))
                .ok_or_else(|| "根目录扇区偏移溢出".to_string())?,
        )
        .ok_or_else(|| "根目录扇区偏移溢出".to_string())?;
    let start = layout
        .data_offset
        .checked_add(
            first_root_sector
                .checked_mul(bps)
                .ok_or_else(|| "根目录偏移溢出".to_string())?,
        )
        .ok_or_else(|| "根目录偏移溢出".to_string())?;
    (0..usize::from(layout.fat12.root_entries))
        .map(|index| {
            start
                .checked_add(index * 32)
                .and_then(|offset| u64::try_from(offset).ok())
                .ok_or_else(|| "根目录项偏移溢出".to_string())
        })
        .collect()
}

fn directory_chain_slots(layout: &DiskLayout, chain: &[u16]) -> Result<Vec<u64>> {
    if !layout.cluster_bytes.is_multiple_of(32) {
        return Err("目录簇大小不是 32 字节目录项的整数倍".to_string());
    }
    let mut slots = Vec::with_capacity(chain.len() * (layout.cluster_bytes / 32));
    for &cluster in chain {
        let start = cluster_offset(layout, cluster)?;
        for relative in (0..layout.cluster_bytes).step_by(32) {
            slots.push(u64::try_from(start + relative).map_err(|_| "目录项偏移溢出".to_string())?);
        }
    }
    Ok(slots)
}

fn active_directory_offsets(source: &[u8], slots: &[u64]) -> Result<HashSet<u64>> {
    let mut active = HashSet::new();
    for &offset in slots {
        let start = usize::try_from(offset).map_err(|_| "目录项偏移过大".to_string())?;
        let entry = source
            .get(start..start.saturating_add(32))
            .ok_or_else(|| format!("目录项偏移 0x{offset:X} 越界"))?;
        if entry[0] == 0x00 {
            break;
        }
        if entry[0] == 0xE5 || entry[0] == b'.' || entry[11] == 0x0F || entry[11] & 0x08 != 0 {
            continue;
        }
        active.insert(offset);
    }
    Ok(active)
}

fn prepare_files(files_root: &Path, archive: &ArchiveManifest) -> Result<Vec<PreparedFile>> {
    archive
        .files
        .iter()
        .map(|entry| {
            let path = join_member_path(files_root, &entry.path)?;
            let data =
                fs::read(&path).map_err(|e| format!("无法读取成员 {}: {e}", path.display()))?;
            if data.len() > u32::MAX as usize {
                return Err(format!("成员 {} 超过 FAT 目录项 u32 大小上限", entry.path));
            }
            let modified = !sha256_hex(&data).eq_ignore_ascii_case(&entry.sha256);
            Ok(PreparedFile {
                data,
                modified,
                desired_chain: Vec::new(),
            })
        })
        .collect()
}

fn allocate_chains(
    source: &[u8],
    layout: &DiskLayout,
    archive: &ArchiveManifest,
    files: &mut [PreparedFile],
) -> Result<()> {
    let fat = first_fat(source, layout);
    let mut reserved = HashSet::new();
    for directory in &archive.directories {
        reserved.extend(directory.cluster_chain.iter().copied());
    }

    for (manifest, prepared) in archive.files.iter().zip(files.iter_mut()) {
        let needed = clusters_for_size(prepared.data.len() as u64, layout.cluster_bytes)?;
        let retained = needed.min(manifest.cluster_chain.len());
        prepared
            .desired_chain
            .extend_from_slice(&manifest.cluster_chain[..retained]);
        for &cluster in &prepared.desired_chain {
            if !reserved.insert(cluster) {
                return Err(format!("簇 {cluster} 被多个保留链占用"));
            }
        }
    }

    let mut available = BTreeSet::new();
    for cluster in 2..=layout.max_cluster {
        if fat12_next(fat, cluster)? == 0 && !reserved.contains(&cluster) {
            available.insert(cluster);
        }
    }
    for (manifest, prepared) in archive.files.iter().zip(files.iter()) {
        for &cluster in manifest
            .cluster_chain
            .iter()
            .skip(prepared.desired_chain.len())
        {
            if !reserved.contains(&cluster) {
                available.insert(cluster);
            }
        }
    }

    for (manifest, prepared) in archive.files.iter().zip(files.iter_mut()) {
        let needed = clusters_for_size(prepared.data.len() as u64, layout.cluster_bytes)?;
        while prepared.desired_chain.len() < needed {
            let Some(cluster) = available.pop_first() else {
                return Err(format!(
                    "磁盘空间不足：{} 需要 {needed} 簇，当前只分配到 {} 簇",
                    manifest.path,
                    prepared.desired_chain.len()
                ));
            };
            if !reserved.insert(cluster) {
                return Err(format!("内部错误：重复分配簇 {cluster}"));
            }
            prepared.desired_chain.push(cluster);
        }
    }
    Ok(())
}

fn rebuild_fat(
    output: &mut [u8],
    layout: &DiskLayout,
    archive: &ArchiveManifest,
    files: &[PreparedFile],
) -> Result<()> {
    let mut fat = first_fat(output, layout).to_vec();
    for (manifest, prepared) in archive.files.iter().zip(files) {
        if manifest.cluster_chain != prepared.desired_chain {
            for &cluster in &manifest.cluster_chain {
                set_fat12(&mut fat, cluster, 0)?;
            }
        }
    }
    for (manifest, prepared) in archive.files.iter().zip(files) {
        if manifest.cluster_chain == prepared.desired_chain {
            continue;
        }
        for (index, &cluster) in prepared.desired_chain.iter().enumerate() {
            let value = prepared
                .desired_chain
                .get(index + 1)
                .copied()
                .unwrap_or(0x0FFF);
            set_fat12(&mut fat, cluster, value)?;
        }
    }
    let copies_to_update = if layout.fat12.fat_copies_identical {
        usize::from(layout.fat12.fat_copies)
    } else {
        1
    };
    for copy in 0..copies_to_update {
        let start = layout
            .fat_offset
            .checked_add(copy * layout.fat_bytes)
            .ok_or_else(|| "FAT 副本偏移溢出".to_string())?;
        let end = start + layout.fat_bytes;
        output[start..end].copy_from_slice(&fat);
    }
    Ok(())
}

fn rebuild_files(
    output: &mut [u8],
    layout: &DiskLayout,
    archive: &ArchiveManifest,
    files: &[PreparedFile],
) -> Result<()> {
    for (manifest, prepared) in archive.files.iter().zip(files) {
        if !prepared.modified {
            continue;
        }
        let mut consumed = 0usize;
        for &cluster in &prepared.desired_chain {
            let offset = cluster_offset(layout, cluster)?;
            let count = (prepared.data.len() - consumed).min(layout.cluster_bytes);
            output[offset..offset + count]
                .copy_from_slice(&prepared.data[consumed..consumed + count]);
            consumed += count;
        }
        if consumed != prepared.data.len() {
            return Err(format!("{} 的簇链容量不足", manifest.path));
        }
        let entry = usize::try_from(manifest.directory_entry_offset)
            .map_err(|_| format!("{} 的目录项偏移过大", manifest.path))?;
        let start_cluster = prepared.desired_chain.first().copied().unwrap_or(0);
        put_u16(output, entry + 26, start_cluster)?;
        put_u32(output, entry + 28, prepared.data.len() as u32)?;
    }
    Ok(())
}

fn validate_rebuilt(
    output: &[u8],
    source: &[u8],
    layout: &DiskLayout,
    archive: &ArchiveManifest,
    files: &[PreparedFile],
) -> Result<()> {
    let reparsed = parse_layout(Path::new("<rebuilt>"), output)?;
    let mut expected_fat12 = layout.fat12.clone();
    expected_fat12.fat_copies_identical = reparsed.fat12.fat_copies_identical;
    if reparsed.fdi != layout.fdi || reparsed.fat12 != expected_fat12 {
        return Err("重建后 FDI/FAT12 布局发生意外变化".to_string());
    }
    let fat = first_fat(output, layout);
    for copy in 1..usize::from(layout.fat12.fat_copies) {
        let start = layout.fat_offset + copy * layout.fat_bytes;
        let end = start + layout.fat_bytes;
        if layout.fat12.fat_copies_identical {
            if output[start..end] != *fat {
                return Err(format!("重建后第 {} 份 FAT 不一致", copy + 1));
            }
        } else if output[start..end] != source[start..end] {
            return Err(format!("重建意外改写了第 {} 份备用 FAT", copy + 1));
        }
    }
    let mut seen = HashSet::new();
    for directory in &archive.directories {
        validate_chain(
            fat,
            layout.max_cluster,
            directory.start_cluster,
            &directory.cluster_chain,
            &directory.path,
            &mut seen,
        )?;
    }
    for (manifest, prepared) in archive.files.iter().zip(files) {
        let entry = usize::try_from(manifest.directory_entry_offset)
            .map_err(|_| format!("{} 的目录项偏移过大", manifest.path))?;
        let expected_start = prepared.desired_chain.first().copied().unwrap_or(0);
        if read_u16(output, entry + 26)? != expected_start
            || read_u32(output, entry + 28)? != prepared.data.len() as u32
        {
            return Err(format!("{} 的重建目录项不一致", manifest.path));
        }
        if !prepared.desired_chain.is_empty() {
            validate_chain(
                fat,
                layout.max_cluster,
                expected_start,
                &prepared.desired_chain,
                &manifest.path,
                &mut seen,
            )?;
        }
        let actual =
            read_file_from_chain(output, layout, &prepared.desired_chain, prepared.data.len())?;
        if actual != prepared.data {
            return Err(format!("{} 的重建内容校验失败", manifest.path));
        }
    }
    Ok(())
}

fn parse_layout(path: &Path, bytes: &[u8]) -> Result<DiskLayout> {
    if bytes.len() < 36 {
        return Err(format!("{}: FDI 头被截断", path.display()));
    }
    let fdi = FdiHeader {
        unknown_0x00: read_u32(bytes, 0)?,
        disk_type: read_u32(bytes, 4)?,
        data_offset: read_u32(bytes, 8)?,
        data_size: read_u32(bytes, 12)?,
        sector_size: read_u32(bytes, 16)?,
        sectors_per_track: read_u32(bytes, 20)?,
        heads: read_u32(bytes, 24)?,
        cylinders: read_u32(bytes, 28)?,
    };
    let data_offset = usize::try_from(fdi.data_offset).map_err(|_| "FDI 数据偏移过大")?;
    let data_size = usize::try_from(fdi.data_size).map_err(|_| "FDI 数据大小过大")?;
    let data_end = data_offset
        .checked_add(data_size)
        .ok_or_else(|| "FDI 数据范围溢出".to_string())?;
    if data_offset < 32 || data_end != bytes.len() {
        return Err(format!("{}: FDI 声明范围与实际长度不一致", path.display()));
    }
    let geometry = [
        fdi.sector_size,
        fdi.sectors_per_track,
        fdi.heads,
        fdi.cylinders,
    ]
    .into_iter()
    .try_fold(1u64, |value, item| {
        value
            .checked_mul(u64::from(item))
            .ok_or_else(|| "FDI 几何容量溢出".to_string())
    })?;
    if geometry != u64::from(fdi.data_size) {
        return Err(format!("{}: FDI 几何容量不一致", path.display()));
    }
    let boot = &bytes[data_offset..];
    let bytes_per_sector = read_u16(boot, 11)?;
    let sectors_per_cluster = *boot.get(13).ok_or("BPB 被截断")?;
    let reserved_sectors = read_u16(boot, 14)?;
    let fat_copies = *boot.get(16).ok_or("BPB 被截断")?;
    let root_entries = read_u16(boot, 17)?;
    let total16 = read_u16(boot, 19)?;
    let media_descriptor = *boot.get(21).ok_or("BPB 被截断")?;
    let sectors_per_fat = read_u16(boot, 22)?;
    let sectors_per_track = read_u16(boot, 24)?;
    let heads = read_u16(boot, 26)?;
    let total_sectors = if total16 == 0 {
        read_u32(boot, 32)?
    } else {
        u32::from(total16)
    };
    if !(128..=4096).contains(&bytes_per_sector)
        || !bytes_per_sector.is_power_of_two()
        || sectors_per_cluster == 0
        || !sectors_per_cluster.is_power_of_two()
        || reserved_sectors == 0
        || fat_copies == 0
        || root_entries == 0
        || sectors_per_fat == 0
    {
        return Err(format!("{}: FAT12 BPB 字段无效", path.display()));
    }
    if u32::from(bytes_per_sector) != fdi.sector_size
        || u32::from(sectors_per_track) != fdi.sectors_per_track
        || u32::from(heads) != fdi.heads
        || u64::from(total_sectors) * u64::from(bytes_per_sector) != u64::from(fdi.data_size)
    {
        return Err(format!("{}: FDI 几何与 BPB 不一致", path.display()));
    }
    let root_directory_sectors =
        (u32::from(root_entries) * 32).div_ceil(u32::from(bytes_per_sector));
    let first_root_sector = u32::from(reserved_sectors)
        .checked_add(u32::from(fat_copies) * u32::from(sectors_per_fat))
        .ok_or_else(|| "FAT 布局溢出".to_string())?;
    let first_data_sector = first_root_sector
        .checked_add(root_directory_sectors)
        .ok_or_else(|| "FAT 数据区偏移溢出".to_string())?;
    if first_data_sector >= total_sectors {
        return Err(format!("{}: FAT 数据区超出磁盘", path.display()));
    }
    let data_clusters = (total_sectors - first_data_sector) / u32::from(sectors_per_cluster);
    if data_clusters == 0 || data_clusters >= 4085 {
        return Err(format!("{}: 数据簇数不属于 FAT12", path.display()));
    }
    let max_cluster = u16::try_from(data_clusters + 1).map_err(|_| "FAT12 簇号溢出")?;
    let bps = usize::from(bytes_per_sector);
    let fat_bytes = usize::from(sectors_per_fat)
        .checked_mul(bps)
        .ok_or_else(|| "FAT 大小溢出".to_string())?;
    let fat_offset = data_offset
        .checked_add(usize::from(reserved_sectors) * bps)
        .ok_or_else(|| "FAT 偏移溢出".to_string())?;
    let fat_end = fat_offset
        .checked_add(fat_bytes)
        .ok_or_else(|| "FAT 范围溢出".to_string())?;
    if fat_end > bytes.len() {
        return Err(format!("{}: FAT 被截断", path.display()));
    }
    let fat = &bytes[fat_offset..fat_end];
    if fat.len() < 3
        || fat[0] != media_descriptor
        || fat[1] != 0xFF
        || fat[2] != 0xFF
        || (fat_bytes * 2) / 3 <= usize::from(max_cluster)
    {
        return Err(format!("{}: FAT12 保留项或容量无效", path.display()));
    }
    let mut fat_copies_identical = true;
    for copy in 1..usize::from(fat_copies) {
        let start = fat_offset
            .checked_add(
                copy.checked_mul(fat_bytes)
                    .ok_or_else(|| "FAT 副本偏移溢出".to_string())?,
            )
            .ok_or_else(|| "FAT 副本偏移溢出".to_string())?;
        let end = start
            .checked_add(fat_bytes)
            .ok_or_else(|| "FAT 副本范围溢出".to_string())?;
        if end > bytes.len() {
            return Err(format!("{}: 第 {} 份 FAT 被截断", path.display(), copy + 1));
        }
        fat_copies_identical &= bytes[start..end] == *fat;
    }
    let fat12 = Fat12Info {
        bytes_per_sector,
        sectors_per_cluster,
        reserved_sectors,
        fat_copies,
        root_entries,
        total_sectors,
        media_descriptor,
        sectors_per_fat,
        sectors_per_track,
        heads,
        root_directory_sectors,
        first_data_sector,
        data_clusters,
        fat_copies_identical,
    };
    Ok(DiskLayout {
        fdi,
        fat12,
        data_offset,
        fat_offset,
        fat_bytes,
        cluster_bytes: bps * usize::from(sectors_per_cluster),
        max_cluster,
    })
}

fn validate_files_root(root: &Path, archive: &ArchiveManifest) -> Result<()> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|e| format!("无法读取成员目录 {}: {e}", root.display()))?;
    if is_reparse_point(&metadata) || metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("成员根路径必须是普通目录: {}", root.display()));
    }
    let expected_files: HashSet<String> =
        archive.files.iter().map(|item| item.path.clone()).collect();
    let expected_dirs: HashSet<String> = archive
        .directories
        .iter()
        .map(|item| item.path.clone())
        .collect();
    let mut actual_files = HashSet::new();
    let mut actual_dirs = HashSet::new();
    scan_tree(root, root, 0, &mut actual_files, &mut actual_dirs)?;
    let missing: Vec<_> = expected_files.difference(&actual_files).cloned().collect();
    let extra: Vec<_> = actual_files.difference(&expected_files).cloned().collect();
    let missing_dirs: Vec<_> = expected_dirs.difference(&actual_dirs).cloned().collect();
    let extra_dirs: Vec<_> = actual_dirs.difference(&expected_dirs).cloned().collect();
    if !missing.is_empty()
        || !extra.is_empty()
        || !missing_dirs.is_empty()
        || !extra_dirs.is_empty()
    {
        return Err(format!(
            "成员树与清单不一致：缺少文件={missing:?}，额外文件={extra:?}，缺少目录={missing_dirs:?}，额外目录={extra_dirs:?}"
        ));
    }
    Ok(())
}

fn scan_tree(
    root: &Path,
    current: &Path,
    depth: usize,
    files: &mut HashSet<String>,
    directories: &mut HashSet<String>,
) -> Result<()> {
    if depth > 64 {
        return Err(format!("成员目录嵌套超过 64 层: {}", current.display()));
    }
    for item in
        fs::read_dir(current).map_err(|e| format!("无法枚举成员目录 {}: {e}", current.display()))?
    {
        let item = item.map_err(|e| format!("无法读取目录项: {e}"))?;
        let path = item.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|e| format!("无法读取成员元数据 {}: {e}", path.display()))?;
        if is_reparse_point(&metadata) || metadata.file_type().is_symlink() {
            return Err(format!(
                "成员树不允许重解析点或符号链接: {}",
                path.display()
            ));
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| "成员路径无法相对化".to_string())?;
        let relative = path_to_manifest_string(relative)?;
        if metadata.is_dir() {
            directories.insert(relative);
            scan_tree(root, &path, depth + 1, files, directories)?;
        } else if metadata.is_file() {
            files.insert(relative);
        } else {
            return Err(format!("成员树含非普通文件: {}", path.display()));
        }
    }
    Ok(())
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

fn validate_directory_entry(
    source: &[u8],
    offset: u64,
    expected: &DirEntryExpectation<'_>,
) -> Result<()> {
    let offset =
        usize::try_from(offset).map_err(|_| format!("{}: 目录项偏移过大", expected.path))?;
    let entry = source
        .get(offset..offset.saturating_add(32))
        .ok_or_else(|| format!("{}: 目录项偏移越界 0x{offset:X}", expected.path))?;
    let raw_name = decode_hex_11(expected.raw_name_hex)?;
    if entry[..11] != raw_name
        || entry[11] != expected.attributes
        || read_u16(entry, 22)? != expected.dos_time
        || read_u16(entry, 24)? != expected.dos_date
        || read_u16(entry, 26)? != expected.start_cluster
        || expected
            .size
            .is_some_and(|size| read_u32(entry, 28).ok() != Some(size))
    {
        return Err(format!(
            "{}: 清单目录项字段与源盘 0x{offset:X} 不一致",
            expected.path
        ));
    }
    Ok(())
}

fn validate_chain(
    fat: &[u8],
    max_cluster: u16,
    start: u16,
    expected: &[u16],
    owner: &str,
    owners: &mut HashSet<u16>,
) -> Result<()> {
    if expected.is_empty() || start != expected[0] {
        return Err(format!("{owner}: 起始簇与清单链不一致"));
    }
    let mut current = start;
    for (index, &expected_cluster) in expected.iter().enumerate() {
        if current != expected_cluster || current < 2 || current > max_cluster {
            return Err(format!("{owner}: 清单簇链在第 {index} 项不一致"));
        }
        if !owners.insert(current) {
            return Err(format!("{owner}: 簇 {current} 与其他成员交叉链接"));
        }
        let next = fat12_next(fat, current)?;
        if index + 1 == expected.len() {
            if !(0xFF8..=0xFFF).contains(&next) {
                return Err(format!("{owner}: 簇链末尾不是 EOC"));
            }
        } else {
            current = next;
        }
    }
    Ok(())
}

fn first_fat<'a>(bytes: &'a [u8], layout: &DiskLayout) -> &'a [u8] {
    &bytes[layout.fat_offset..layout.fat_offset + layout.fat_bytes]
}

fn fat12_next(fat: &[u8], cluster: u16) -> Result<u16> {
    let offset = usize::from(cluster) * 3 / 2;
    let pair = fat
        .get(offset..offset.saturating_add(2))
        .ok_or_else(|| format!("FAT12 项 {cluster} 越界"))?;
    let word = u16::from(pair[0]) | (u16::from(pair[1]) << 8);
    Ok(if cluster & 1 == 0 {
        word & 0x0FFF
    } else {
        word >> 4
    })
}

fn set_fat12(fat: &mut [u8], cluster: u16, value: u16) -> Result<()> {
    if value > 0x0FFF {
        return Err(format!("FAT12 值超出 12 位: 0x{value:X}"));
    }
    let offset = usize::from(cluster) * 3 / 2;
    if offset + 2 > fat.len() {
        return Err(format!("FAT12 项 {cluster} 越界"));
    }
    if cluster & 1 == 0 {
        fat[offset] = value as u8;
        fat[offset + 1] = (fat[offset + 1] & 0xF0) | ((value >> 8) as u8 & 0x0F);
    } else {
        fat[offset] = (fat[offset] & 0x0F) | ((value << 4) as u8 & 0xF0);
        fat[offset + 1] = (value >> 4) as u8;
    }
    Ok(())
}

fn cluster_offset(layout: &DiskLayout, cluster: u16) -> Result<usize> {
    if cluster < 2 || cluster > layout.max_cluster {
        return Err(format!("簇 {cluster} 超出有效范围"));
    }
    let sector = u32::from(cluster - 2)
        .checked_mul(u32::from(layout.fat12.sectors_per_cluster))
        .and_then(|value| value.checked_add(layout.fat12.first_data_sector))
        .ok_or_else(|| format!("簇 {cluster} 偏移溢出"))?;
    usize::try_from(sector)
        .map_err(|_| format!("簇 {cluster} 偏移过大"))?
        .checked_mul(usize::from(layout.fat12.bytes_per_sector))
        .and_then(|value| value.checked_add(layout.data_offset))
        .ok_or_else(|| format!("簇 {cluster} 字节偏移溢出"))
}

fn read_file_from_chain(
    bytes: &[u8],
    layout: &DiskLayout,
    chain: &[u16],
    size: usize,
) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(chain.len() * layout.cluster_bytes);
    for &cluster in chain {
        let offset = cluster_offset(layout, cluster)?;
        output.extend_from_slice(&bytes[offset..offset + layout.cluster_bytes]);
    }
    if size > output.len() {
        return Err("簇链容量小于文件大小".to_string());
    }
    output.truncate(size);
    Ok(output)
}

fn clusters_for_size(size: u64, cluster_bytes: usize) -> Result<usize> {
    if size == 0 {
        return Ok(0);
    }
    usize::try_from(size.div_ceil(cluster_bytes as u64))
        .map_err(|_| "文件需要的簇数过大".to_string())
}

fn count_free_clusters(fat: &[u8], max_cluster: u16) -> Result<usize> {
    (2..=max_cluster).try_fold(0usize, |count, cluster| {
        Ok(count + usize::from(fat12_next(fat, cluster)? == 0))
    })
}

fn validate_member_path(path: &str) -> Result<()> {
    if path.is_empty() || path.starts_with('/') || path.ends_with('/') || path.contains('\\') {
        return Err(format!("不安全的清单成员路径: {path:?}"));
    }
    for segment in path.split('/') {
        validate_segment(segment)?;
    }
    Ok(())
}

fn validate_relative_role_path(path: &str, allow_dot: bool) -> Result<()> {
    if allow_dot && path == "." {
        return Ok(());
    }
    validate_member_path(path)
}

fn validate_segment(segment: &str) -> Result<()> {
    if segment.is_empty() || segment == "." || segment == ".." {
        return Err(format!("不安全的路径段: {segment:?}"));
    }
    if segment.ends_with(' ') || segment.ends_with('.') {
        return Err(format!("Windows 路径段不能以空格或点结尾: {segment}"));
    }
    if segment
        .chars()
        .any(|ch| ch < ' ' || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'))
    {
        return Err(format!("Windows 路径段含非法字符: {segment}"));
    }
    Ok(())
}

fn join_member_path(root: &Path, relative: &str) -> Result<PathBuf> {
    validate_member_path(relative)?;
    let mut output = root.to_path_buf();
    for segment in relative.split('/') {
        output.push(segment);
    }
    Ok(output)
}

fn path_to_manifest_string(path: &Path) -> Result<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => parts.push(
                value
                    .to_str()
                    .ok_or_else(|| format!("成员路径无法表示为 Unicode: {}", path.display()))?,
            ),
            _ => return Err(format!("成员路径含非法组件: {}", path.display())),
        }
    }
    Ok(parts.join("/"))
}

fn insert_unique(paths: &mut HashSet<String>, path: &str) -> Result<()> {
    let key = path.to_uppercase();
    if !paths.insert(key) {
        return Err(format!("清单路径发生大小写不敏感重复: {path}"));
    }
    Ok(())
}

fn insert_offset(offsets: &mut HashSet<u64>, offset: u64, path: &str) -> Result<()> {
    if !offsets.insert(offset) {
        return Err(format!("清单目录项偏移重复: {path} @ 0x{offset:X}"));
    }
    Ok(())
}

fn validate_hash(hash: &str, field: &str) -> Result<()> {
    if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{field} 不是 64 位十六进制 SHA-256"));
    }
    Ok(())
}

fn decode_hex_11(text: &str) -> Result<[u8; 11]> {
    if text.len() != 22 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("raw_short_name_hex 无效: {text:?}"));
    }
    let mut output = [0u8; 11];
    for (index, byte) in output.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)
            .map_err(|e| format!("raw_short_name_hex 无效: {e}"))?;
    }
    Ok(output)
}

fn validate_disjoint_output(output: &Path, inputs: &[(&str, &Path)]) -> Result<()> {
    let output_components = comparable_path_components(output)?;
    for (role, input) in inputs {
        let input_components = comparable_path_components(input)?;
        if component_prefix(&output_components, &input_components)
            || component_prefix(&input_components, &output_components)
        {
            return Err(format!(
                "输出路径不得与{role}重叠: output={} {role}={}",
                output.display(),
                input.display()
            ));
        }
    }
    Ok(())
}

fn comparable_path_components(path: &Path) -> Result<Vec<String>> {
    Ok(absolute_lexical(path)?
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
        .collect())
}

fn component_prefix(prefix: &[String], path: &[String]) -> bool {
    prefix.len() <= path.len() && prefix.iter().zip(path).all(|(left, right)| left == right)
}

fn validate_output_path(output: &Path, overwrite: bool) -> Result<()> {
    if output.exists() {
        let metadata = fs::symlink_metadata(output)
            .map_err(|e| format!("无法读取输出元数据 {}: {e}", output.display()))?;
        if is_reparse_point(&metadata) || metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!("输出必须是普通文件路径: {}", output.display()));
        }
        if !overwrite {
            return Err(format!(
                "输出已存在；请修改路径或显式指定 --overwrite: {}",
                output.display()
            ));
        }
    }
    let parent = output
        .parent()
        .ok_or_else(|| format!("输出路径没有父目录: {}", output.display()))?;
    if !parent.exists() || !parent.is_dir() {
        return Err(format!("输出父目录不存在: {}", parent.display()));
    }
    Ok(())
}

fn absolute_lexical(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path).map_err(|e| format!("无法解析路径 {}: {e}", path.display()));
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("路径没有父目录: {}", path.display()))?;
    let parent = fs::canonicalize(parent)
        .map_err(|e| format!("无法解析父目录 {}: {e}", parent.display()))?;
    let name = path
        .file_name()
        .ok_or_else(|| format!("路径缺少文件名: {}", path.display()))?;
    Ok(parent.join(name))
}

fn write_outputs_transactional(outputs: &[(&Path, &[u8])], overwrite: bool) -> Result<()> {
    write_outputs_transactional_with(outputs, overwrite, |_, temporary, output| {
        fs::rename(temporary, output)
    })
}

fn write_outputs_transactional_with<F>(
    outputs: &[(&Path, &[u8])],
    overwrite: bool,
    mut commit_rename: F,
) -> Result<()>
where
    F: FnMut(usize, &Path, &Path) -> io::Result<()>,
{
    for (output, _) in outputs {
        output
            .parent()
            .ok_or_else(|| format!("输出路径没有父目录: {}", output.display()))?;
        output
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("输出文件名无法表示为 Unicode: {}", output.display()))?;
    }

    let mut staged = Vec::with_capacity(outputs.len());
    for (output, bytes) in outputs {
        let parent = output.parent().expect("all output parents were validated");
        let name = output
            .file_name()
            .and_then(|value| value.to_str())
            .expect("all output names were validated");
        let (temporary, mut file) = match create_unique_temporary(
            parent,
            &format!(".{name}.batch-tmp-{}", std::process::id()),
        ) {
            Ok(staged_file) => staged_file,
            Err(error) => return fail_output_transaction(&mut staged, error),
        };
        staged.push(StagedOutput {
            temporary,
            output: (*output).to_path_buf(),
            backup: None,
            committed: false,
        });
        let current = staged.last().expect("staged output was just inserted");
        let write_result = (|| -> Result<()> {
            file.write_all(bytes).map_err(|e| {
                format!("无法写入批量临时输出 {}: {e}", current.temporary.display())
            })?;
            file.sync_all().map_err(|e| {
                format!("无法刷新批量临时输出 {}: {e}", current.temporary.display())
            })?;
            Ok(())
        })();
        drop(file);
        if let Err(error) = write_result {
            return fail_output_transaction(&mut staged, error);
        }
    }

    for index in 0..staged.len() {
        let output = staged[index].output.clone();
        if output.exists() {
            let metadata = match fs::symlink_metadata(&output) {
                Ok(metadata) => metadata,
                Err(error) => {
                    return fail_output_transaction(
                        &mut staged,
                        format!("无法读取提交目标元数据 {}: {error}", output.display()),
                    );
                }
            };
            if is_reparse_point(&metadata)
                || metadata.file_type().is_symlink()
                || !metadata.is_file()
            {
                return fail_output_transaction(
                    &mut staged,
                    format!("批量提交目标必须是普通文件: {}", output.display()),
                );
            }
            if !overwrite {
                return fail_output_transaction(
                    &mut staged,
                    format!("输出在预检后出现且未授权覆盖: {}", output.display()),
                );
            }
            let parent = output.parent().expect("validated output has a parent");
            let backup = match unique_sibling(
                parent,
                &format!(".fdi-pack-batch-backup-{}", std::process::id()),
            ) {
                Ok(backup) => backup,
                Err(error) => return fail_output_transaction(&mut staged, error),
            };
            if let Err(error) = fs::rename(&output, &backup) {
                return fail_output_transaction(
                    &mut staged,
                    format!("无法备份现有批量输出 {}: {error}", output.display()),
                );
            }
            staged[index].backup = Some(backup);
        }

        if let Err(error) = commit_rename(index, &staged[index].temporary, &staged[index].output) {
            return fail_output_transaction(
                &mut staged,
                format!("提交批量输出失败 {}: {error}", output.display()),
            );
        }
        staged[index].committed = true;
    }

    let mut cleanup_errors = Vec::new();
    for item in &mut staged {
        if let Some(backup) = item.backup.take() {
            if let Err(error) = fs::remove_file(&backup) {
                cleanup_errors.push(format!("无法删除批量备份 {}: {error}", backup.display()));
            }
        }
    }
    if cleanup_errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "批量输出已提交，但备份清理失败: {}",
            cleanup_errors.join("; ")
        ))
    }
}

fn fail_output_transaction(staged: &mut [StagedOutput], error: String) -> Result<()> {
    let rollback_errors = rollback_staged_outputs(staged);
    if rollback_errors.is_empty() {
        Err(format!("{error}；已回滚全部批量输出"))
    } else {
        Err(format!(
            "{error}；批量回滚不完整: {}",
            rollback_errors.join("; ")
        ))
    }
}

fn rollback_staged_outputs(staged: &mut [StagedOutput]) -> Vec<String> {
    let mut errors = Vec::new();
    for item in staged.iter_mut().rev() {
        if (item.committed || item.backup.is_some()) && item.output.exists() {
            if let Err(error) = fs::remove_file(&item.output) {
                errors.push(format!(
                    "无法移除已提交输出 {}: {error}",
                    item.output.display()
                ));
            } else {
                item.committed = false;
            }
        }
        if let Some(backup) = item.backup.as_ref() {
            if item.output.exists() {
                errors.push(format!(
                    "无法恢复备份，目标仍存在: {}",
                    item.output.display()
                ));
            } else if let Err(error) = fs::rename(backup, &item.output) {
                errors.push(format!(
                    "无法恢复批量备份 {} -> {}: {error}",
                    backup.display(),
                    item.output.display()
                ));
            } else {
                item.backup = None;
            }
        }
        if item.temporary.exists() {
            if let Err(error) = fs::remove_file(&item.temporary) {
                errors.push(format!(
                    "无法清理批量临时文件 {}: {error}",
                    item.temporary.display()
                ));
            }
        }
    }
    errors
}

fn write_output(path: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| "输出路径没有父目录".to_string())?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "输出文件名无法表示为 Unicode".to_string())?;
    let (temporary, mut file) =
        create_unique_temporary(parent, &format!(".{name}.tmp-{}", std::process::id()))?;
    let result = (|| -> Result<()> {
        file.write_all(bytes)
            .map_err(|e| format!("无法写入临时输出 {}: {e}", temporary.display()))?;
        file.sync_all()
            .map_err(|e| format!("无法刷新临时输出 {}: {e}", temporary.display()))?;
        drop(file);
        commit_file(&temporary, path, overwrite)
    })();
    if result.is_err() && temporary.exists() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn create_unique_temporary(parent: &Path, base: &str) -> Result<(PathBuf, fs::File)> {
    for suffix in 0..1000 {
        let name = if suffix == 0 {
            base.to_string()
        } else {
            format!("{base}-{suffix}")
        };
        let candidate = parent.join(name);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => return Ok((candidate, file)),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!("无法创建临时输出 {}: {error}", candidate.display()));
            }
        }
    }
    Err(format!("无法在 {} 创建唯一临时输出", parent.display()))
}

fn commit_file(temporary: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return fs::rename(temporary, output)
            .map_err(|e| format!("无法提交输出 {}: {e}", output.display()));
    }
    if !overwrite {
        return Err(format!("输出已存在: {}", output.display()));
    }
    let parent = output
        .parent()
        .ok_or_else(|| "输出路径没有父目录".to_string())?;
    let backup = unique_sibling(parent, &format!(".fdi-pack-backup-{}", std::process::id()))?;
    fs::rename(output, &backup)
        .map_err(|e| format!("无法暂存现有输出 {}: {e}", output.display()))?;
    if let Err(error) = fs::rename(temporary, output) {
        let rollback = fs::rename(&backup, output);
        return match rollback {
            Ok(()) => Err(format!("提交输出失败，已恢复旧文件: {error}")),
            Err(rollback_error) => Err(format!(
                "提交输出失败且恢复失败，备份位于 {}: {error}; {rollback_error}",
                backup.display()
            )),
        };
    }
    fs::remove_file(&backup)
        .map_err(|e| format!("输出已提交，但无法删除备份 {}: {e}", backup.display()))?;
    Ok(())
}

fn unique_sibling(parent: &Path, base: &str) -> Result<PathBuf> {
    for suffix in 0..1000 {
        let name = if suffix == 0 {
            base.to_string()
        } else {
            format!("{base}-{suffix}")
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format!("无法在 {} 创建唯一临时文件名", parent.display()))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let slice = bytes
        .get(offset..offset.saturating_add(2))
        .ok_or_else(|| format!("读取 0x{offset:X} 处 u16 越界"))?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let slice = bytes
        .get(offset..offset.saturating_add(4))
        .ok_or_else(|| format!("读取 0x{offset:X} 处 u32 越界"))?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) -> Result<()> {
    let target = bytes
        .get_mut(offset..offset.saturating_add(2))
        .ok_or_else(|| format!("写入 0x{offset:X} 处 u16 越界"))?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<()> {
    let target = bytes
        .get_mut(offset..offset.saturating_add(4))
        .ok_or_else(|| format!("写入 0x{offset:X} 处 u32 越界"))?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = String::with_capacity(64);
    for &byte in digest.as_slice() {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0F)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const HEADER: usize = 0x100;
    const SECTOR: usize = 1024;
    const SECTORS: usize = 16;

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "aitsuno-fdi-pack-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn sample_fdi() -> Vec<u8> {
        let mut bytes = vec![0xCC; HEADER + SECTOR * SECTORS];
        put_u32(&mut bytes, 0, 0).unwrap();
        put_u32(&mut bytes, 4, 0x90).unwrap();
        put_u32(&mut bytes, 8, HEADER as u32).unwrap();
        put_u32(&mut bytes, 12, (SECTOR * SECTORS) as u32).unwrap();
        put_u32(&mut bytes, 16, SECTOR as u32).unwrap();
        put_u32(&mut bytes, 20, 4).unwrap();
        put_u32(&mut bytes, 24, 1).unwrap();
        put_u32(&mut bytes, 28, 4).unwrap();
        let boot = HEADER;
        bytes[boot..boot + SECTOR].fill(0);
        bytes[boot..boot + 3].copy_from_slice(&[0xEB, 0x1C, 0x90]);
        put_u16(&mut bytes, boot + 11, SECTOR as u16).unwrap();
        bytes[boot + 13] = 1;
        put_u16(&mut bytes, boot + 14, 1).unwrap();
        bytes[boot + 16] = 2;
        put_u16(&mut bytes, boot + 17, 32).unwrap();
        put_u16(&mut bytes, boot + 19, SECTORS as u16).unwrap();
        bytes[boot + 21] = 0xFE;
        put_u16(&mut bytes, boot + 22, 1).unwrap();
        put_u16(&mut bytes, boot + 24, 4).unwrap();
        put_u16(&mut bytes, boot + 26, 1).unwrap();
        let mut fat = vec![0u8; SECTOR];
        fat[..3].copy_from_slice(&[0xFE, 0xFF, 0xFF]);
        set_fat12(&mut fat, 2, 3).unwrap();
        set_fat12(&mut fat, 3, 0xFFF).unwrap();
        bytes[HEADER + SECTOR..HEADER + SECTOR * 2].copy_from_slice(&fat);
        bytes[HEADER + SECTOR * 2..HEADER + SECTOR * 3].copy_from_slice(&fat);
        let root = HEADER + SECTOR * 3;
        bytes[root..root + SECTOR].fill(0);
        bytes[root..root + 11].copy_from_slice(b"HELLO   TXT");
        bytes[root + 11] = 0x20;
        put_u16(&mut bytes, root + 22, 1).unwrap();
        put_u16(&mut bytes, root + 24, 2).unwrap();
        put_u16(&mut bytes, root + 26, 2).unwrap();
        put_u32(&mut bytes, root + 28, 1500).unwrap();
        let cluster2 = HEADER + SECTOR * 4;
        for index in 0..1500 {
            bytes[cluster2 + index] = (index % 251) as u8;
        }
        bytes
    }

    fn setup(label: &str) -> (PathBuf, PathBuf, PathBuf, Vec<u8>) {
        let base = temp_dir(label);
        fs::create_dir(&base).expect("base");
        let source_path = base.join("source.fdi");
        let files = base.join("arbitrary files");
        fs::create_dir(&files).expect("files");
        let source = sample_fdi();
        fs::write(&source_path, &source).expect("source");
        let data = source[HEADER + SECTOR * 4..HEADER + SECTOR * 4 + 1500].to_vec();
        fs::write(files.join("HELLO.TXT"), &data).expect("member");
        let layout = parse_layout(&source_path, &source).expect("layout");
        let archive = ArchiveManifest {
            source_file: "original-name.fdi".to_string(),
            source_sha256: sha256_hex(&source),
            output_dir: "renamable-role".to_string(),
            archive_bytes: source.len() as u64,
            fdi: layout.fdi.clone(),
            fat12: layout.fat12.clone(),
            volume_labels: Vec::new(),
            directories: Vec::new(),
            files: vec![FileManifest {
                path: "HELLO.TXT".to_string(),
                raw_short_name_hex: "48454C4C4F202020545854".to_string(),
                attributes: 0x20,
                directory_entry_offset: (HEADER + SECTOR * 3) as u64,
                start_cluster: 2,
                size: 1500,
                cluster_chain: vec![2, 3],
                dos_time: 1,
                dos_date: 2,
                sha256: sha256_hex(&data),
            }],
            skipped_deleted_entries: 0,
            skipped_lfn_entries: 0,
        };
        let manifest = WorkspaceManifest {
            _format: WORKSPACE_FORMAT.to_string(),
            tool_version: "test".to_string(),
            role_paths: RolePaths {
                unpacked_root: ".".to_string(),
            },
            archives: vec![archive],
        };
        let manifest_path = base.join("renamed.json");
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("json"),
        )
        .expect("manifest");
        (base, source_path, manifest_path, source)
    }

    fn setup_batch(label: &str) -> (PathBuf, PathBuf, PathBuf, PathBuf, Vec<u8>) {
        let (base, source_path, manifest_path, source) = setup(label);
        let source_root = base.join("original images");
        fs::create_dir(&source_root).unwrap();
        let renamed_source = source_root.join("arbitrary-image.bin");
        fs::rename(source_path, &renamed_source).unwrap();

        let unpacked_root = base.join("renamed unpacked root");
        let workspace_parent = unpacked_root.join("nested project");
        let managed_root = workspace_parent.join("payload role");
        let files_root = managed_root.join("renamed member role");
        fs::create_dir_all(&managed_root).unwrap();
        fs::rename(base.join("arbitrary files"), &files_root).unwrap();

        let mut workspace: WorkspaceManifest =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        workspace.role_paths.unpacked_root = "payload role".to_string();
        workspace.archives[0].source_file = "logical output/original disk.fdi".to_string();
        workspace.archives[0].output_dir = "renamed member role".to_string();
        let workspace_path = workspace_parent.join("renamed manifest.data");
        fs::write(
            &workspace_path,
            serde_json::to_vec_pretty(&workspace).unwrap(),
        )
        .unwrap();
        fs::remove_file(manifest_path).unwrap();
        (base, source_root, unpacked_root, renamed_source, source)
    }

    #[test]
    fn unchanged_pack_is_byte_exact() {
        let (base, source, manifest, expected) = setup("unchanged");
        let output = base.join("output.fdi");
        let report = pack_image(
            &source,
            &manifest,
            &base.join("arbitrary files"),
            &output,
            false,
        )
        .expect("pack");
        assert_eq!(report.modified_files, 0);
        assert_eq!(fs::read(output).unwrap(), expected);
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn recursive_batch_uses_manifest_roles_and_stable_output_mapping() {
        let (base, source_root, unpacked_root, _, expected) = setup_batch("recursive-batch");
        let extra_image = source_root.join("unmatched-valid-image.dat");
        let mut extra = expected.clone();
        extra[40] ^= 1;
        fs::write(extra_image, extra).unwrap();
        let output_root = base.join("packed output");

        let plan = preview_workspace_pack(&source_root, &unpacked_root, &output_root, false)
            .expect("preview");
        assert_eq!(plan.mappings.len(), 1);
        assert_eq!(plan.skipped_source_images, 1);
        assert!(plan.mappings[0]
            .source
            .ends_with("original images/arbitrary-image.bin"));
        assert!(plan.mappings[0]
            .files_root
            .ends_with("nested project/payload role/renamed member role"));
        assert!(plan.mappings[0]
            .output
            .ends_with("nested project/logical output/original disk.fdi"));
        assert!(!output_root.exists());

        let report =
            pack_workspace_tree(&source_root, &unpacked_root, &output_root, false).expect("pack");
        assert_eq!(report.images, 1);
        assert_eq!(report.modified_files, 0);
        assert_eq!(report.skipped_source_images, 1);
        assert_eq!(
            fs::read(
                output_root
                    .join("nested project")
                    .join("logical output")
                    .join("original disk.fdi")
            )
            .unwrap(),
            expected
        );
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn recursive_batch_packs_every_archive_in_the_workspace() {
        let (base, source_root, unpacked_root, _, first_source) = setup_batch("recursive-multiple");
        let workspace_path = unpacked_root
            .join("nested project")
            .join("renamed manifest.data");
        let mut workspace: WorkspaceManifest =
            serde_json::from_slice(&fs::read(&workspace_path).unwrap()).unwrap();
        let first_member_root = unpacked_root
            .join("nested project")
            .join("payload role")
            .join("renamed member role");
        let second_member_root = unpacked_root
            .join("nested project")
            .join("payload role")
            .join("second member role");
        fs::create_dir(&second_member_root).unwrap();
        fs::copy(
            first_member_root.join("HELLO.TXT"),
            second_member_root.join("HELLO.TXT"),
        )
        .unwrap();

        let mut second_source = first_source.clone();
        second_source[40] ^= 1;
        fs::write(
            source_root.join("another arbitrary name.dat"),
            &second_source,
        )
        .unwrap();
        let mut second_archive = workspace.archives[0].clone();
        second_archive.source_file = "second output/translated disk.fdi".to_string();
        second_archive.source_sha256 = sha256_hex(&second_source);
        second_archive.output_dir = "second member role".to_string();
        workspace.archives.push(second_archive);
        fs::write(
            &workspace_path,
            serde_json::to_vec_pretty(&workspace).unwrap(),
        )
        .unwrap();

        let output_root = base.join("all packed disks");
        let report =
            pack_workspace_tree(&source_root, &unpacked_root, &output_root, false).expect("pack");
        assert_eq!(report.images, 2);
        assert_eq!(report.packed_files, 2);
        assert_eq!(report.modified_files, 0);
        assert_eq!(
            fs::read(
                output_root
                    .join("nested project")
                    .join("logical output")
                    .join("original disk.fdi")
            )
            .unwrap(),
            first_source
        );
        assert_eq!(
            fs::read(
                output_root
                    .join("nested project")
                    .join("second output")
                    .join("translated disk.fdi")
            )
            .unwrap(),
            second_source
        );
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn recursive_single_source_uses_explicit_output_file() {
        let (base, _, unpacked_root, source, expected) = setup_batch("recursive-single");
        let output = base.join("explicit output.dat");
        let report = pack_workspace_tree(&source, &unpacked_root, &output, false).expect("pack");
        assert_eq!(report.images, 1);
        assert_eq!(fs::read(output).unwrap(), expected);
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn recursive_preflight_failure_creates_no_output_root() {
        let (base, source_root, unpacked_root, _, _) = setup_batch("recursive-preflight");
        fs::write(
            unpacked_root
                .join("nested project")
                .join("payload role")
                .join("renamed member role")
                .join("EXTRA.BIN"),
            b"unexpected",
        )
        .unwrap();
        let output_root = base.join("must not exist");
        let error =
            pack_workspace_tree(&source_root, &unpacked_root, &output_root, false).unwrap_err();
        assert!(error.contains("额外文件"));
        assert!(!output_root.exists());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn recursive_batch_refuses_existing_target_without_overwrite() {
        let (base, source_root, unpacked_root, _, _) = setup_batch("recursive-existing");
        let output_root = base.join("packed output");
        pack_workspace_tree(&source_root, &unpacked_root, &output_root, false).expect("first");
        let target = output_root
            .join("nested project")
            .join("logical output")
            .join("original disk.fdi");
        let before = fs::read(&target).unwrap();
        let error =
            pack_workspace_tree(&source_root, &unpacked_root, &output_root, false).unwrap_err();
        assert!(error.contains("--overwrite"));
        assert_eq!(fs::read(&target).unwrap(), before);
        pack_workspace_tree(&source_root, &unpacked_root, &output_root, true).expect("overwrite");
        assert_eq!(fs::read(&target).unwrap(), before);
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn batch_commit_failure_restores_previous_outputs_and_cleans_staging() {
        let base = temp_dir("batch-rollback");
        fs::create_dir(&base).unwrap();
        let first = base.join("first.fdi");
        let second = base.join("second.fdi");
        fs::write(&first, b"old first").unwrap();
        fs::write(&second, b"old second").unwrap();
        let payloads = [
            (first.as_path(), b"new first".as_slice()),
            (second.as_path(), b"new second".as_slice()),
        ];

        let error =
            write_outputs_transactional_with(&payloads, true, |index, temporary, output| {
                if index == 1 {
                    Err(io::Error::new(
                        ErrorKind::PermissionDenied,
                        "injected second commit failure",
                    ))
                } else {
                    fs::rename(temporary, output)
                }
            })
            .unwrap_err();

        assert!(error.contains("已回滚全部批量输出"));
        assert_eq!(fs::read(&first).unwrap(), b"old first");
        assert_eq!(fs::read(&second).unwrap(), b"old second");
        let residual: Vec<_> = fs::read_dir(&base)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .filter(|name| name.contains("batch-tmp") || name.contains("batch-backup"))
            .collect();
        assert!(residual.is_empty(), "residual files: {residual:?}");
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn grows_file_and_updates_both_fats() {
        let (base, source, manifest, original) = setup("grow");
        let files = base.join("arbitrary files");
        let data = vec![0x5A; 3000];
        fs::write(files.join("HELLO.TXT"), &data).unwrap();
        let output = base.join("grown.fdi");
        let report = pack_image(&source, &manifest, &files, &output, false).expect("pack");
        assert_eq!(report.modified_files, 1);
        assert_eq!(report.reallocated_files, 1);
        let rebuilt = fs::read(output).unwrap();
        assert_eq!(&rebuilt[..HEADER], &original[..HEADER]);
        assert_eq!(
            &rebuilt[HEADER + SECTOR..HEADER + SECTOR * 2],
            &rebuilt[HEADER + SECTOR * 2..HEADER + SECTOR * 3]
        );
        let layout = parse_layout(Path::new("grown"), &rebuilt).unwrap();
        let fat = first_fat(&rebuilt, &layout);
        assert_eq!(fat12_next(fat, 2).unwrap(), 3);
        assert_eq!(fat12_next(fat, 3).unwrap(), 4);
        assert!((0xFF8..=0xFFF).contains(&fat12_next(fat, 4).unwrap()));
        assert_eq!(
            read_file_from_chain(&rebuilt, &layout, &[2, 3, 4], data.len()).unwrap(),
            data
        );
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn mismatched_backup_fat_is_accepted_and_preserved() {
        let (base, source, manifest, mut original) = setup("mismatched-fat");
        let second_fat = HEADER + SECTOR * 2;
        set_fat12(&mut original[second_fat..second_fat + SECTOR], 5, 0x0FFF).unwrap();
        fs::write(&source, &original).unwrap();

        let layout = parse_layout(&source, &original).expect("mismatched FAT layout");
        assert!(!layout.fat12.fat_copies_identical);
        let mut workspace: WorkspaceManifest =
            serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
        workspace.archives[0].source_sha256 = sha256_hex(&original);
        workspace.archives[0].fat12 = layout.fat12.clone();
        fs::write(&manifest, serde_json::to_vec_pretty(&workspace).unwrap()).unwrap();

        let unchanged_output = base.join("unchanged.fdi");
        let unchanged = pack_image(
            &source,
            &manifest,
            &base.join("arbitrary files"),
            &unchanged_output,
            false,
        )
        .expect("unchanged pack");
        assert!(unchanged.fat_mismatch_preserved);
        assert_eq!(fs::read(&unchanged_output).unwrap(), original);

        fs::write(
            base.join("arbitrary files").join("HELLO.TXT"),
            vec![0x5A; 3000],
        )
        .unwrap();
        let changed_output = base.join("changed.fdi");
        let changed = pack_image(
            &source,
            &manifest,
            &base.join("arbitrary files"),
            &changed_output,
            false,
        )
        .expect("changed pack");
        assert!(changed.fat_mismatch_preserved);
        let rebuilt = fs::read(changed_output).unwrap();
        assert_ne!(
            &rebuilt[HEADER + SECTOR..HEADER + SECTOR * 2],
            &original[HEADER + SECTOR..HEADER + SECTOR * 2]
        );
        assert_eq!(
            &rebuilt[second_fat..second_fat + SECTOR],
            &original[second_fat..second_fat + SECTOR]
        );
        assert!(
            !parse_layout(Path::new("changed"), &rebuilt)
                .unwrap()
                .fat12
                .fat_copies_identical
        );
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn shrinks_file_and_preserves_unused_bytes() {
        let (base, source, manifest, original) = setup("shrink");
        let files = base.join("arbitrary files");
        fs::write(files.join("HELLO.TXT"), b"short").unwrap();
        let output = base.join("short.fdi");
        pack_image(&source, &manifest, &files, &output, false).expect("pack");
        let rebuilt = fs::read(output).unwrap();
        let layout = parse_layout(Path::new("short"), &rebuilt).unwrap();
        let fat = first_fat(&rebuilt, &layout);
        assert!((0xFF8..=0xFFF).contains(&fat12_next(fat, 2).unwrap()));
        assert_eq!(fat12_next(fat, 3).unwrap(), 0);
        let cluster2 = cluster_offset(&layout, 2).unwrap();
        assert_eq!(&rebuilt[cluster2..cluster2 + 5], b"short");
        assert_eq!(rebuilt[cluster2 + 5], original[cluster2 + 5]);
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn rejects_extra_member_before_output() {
        let (base, source, manifest, _) = setup("extra");
        let files = base.join("arbitrary files");
        fs::write(files.join("EXTRA.BIN"), b"x").unwrap();
        let output = base.join("out.fdi");
        let error = pack_image(&source, &manifest, &files, &output, false).unwrap_err();
        assert!(error.contains("额外文件"));
        assert!(!output.exists());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn rejects_source_hash_mismatch() {
        let (base, source, manifest, _) = setup("hash");
        let mut changed = fs::read(&source).unwrap();
        changed[40] ^= 1;
        fs::write(&source, changed).unwrap();
        let output = base.join("out.fdi");
        let error = pack_image(
            &source,
            &manifest,
            &base.join("arbitrary files"),
            &output,
            false,
        )
        .unwrap_err();
        assert!(error.contains("未在工作区清单"));
        assert!(!output.exists());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn rejects_manifest_member_hash_mismatch() {
        let (base, source, manifest, _) = setup("member-hash");
        let mut workspace: WorkspaceManifest =
            serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
        workspace.archives[0].files[0].sha256 = "00".repeat(32);
        fs::write(&manifest, serde_json::to_vec_pretty(&workspace).unwrap()).unwrap();
        let output = base.join("out.fdi");

        let error = pack_image(
            &source,
            &manifest,
            &base.join("arbitrary files"),
            &output,
            false,
        )
        .unwrap_err();
        assert!(error.contains("源盘成员 HELLO.TXT 的 SHA-256 与清单不一致"));
        assert!(!output.exists());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn rejects_output_overlap_with_manifest_or_member_tree() {
        let (base, source, manifest, _) = setup("output-overlap");
        let files = base.join("arbitrary files");
        let manifest_before = fs::read(&manifest).unwrap();

        let error = pack_image(&source, &manifest, &files, &manifest, true).unwrap_err();
        assert!(error.contains("工作区清单重叠"));
        assert_eq!(fs::read(&manifest).unwrap(), manifest_before);

        let nested_output = files.join("new.fdi");
        let error = pack_image(&source, &manifest, &files, &nested_output, false).unwrap_err();
        assert!(error.contains("成员目录重叠"));
        assert!(!nested_output.exists());

        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn rejects_manifest_that_omits_an_active_directory_entry() {
        let (base, source, manifest, _) = setup("missing-entry");
        let mut workspace: WorkspaceManifest =
            serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
        workspace.archives[0].files.clear();
        fs::write(&manifest, serde_json::to_vec_pretty(&workspace).unwrap()).unwrap();
        fs::remove_file(base.join("arbitrary files").join("HELLO.TXT")).unwrap();
        let output = base.join("out.fdi");

        let error = pack_image(
            &source,
            &manifest,
            &base.join("arbitrary files"),
            &output,
            false,
        )
        .unwrap_err();
        assert!(error.contains("清单目录项与源盘活动目录项不完整对应"));
        assert!(!output.exists());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn released_cluster_reuse_is_order_independent() {
        let (base, source_path, manifest_path, mut source) = setup("reuse-order");
        let layout = parse_layout(&source_path, &source).unwrap();
        for copy in 0..usize::from(layout.fat12.fat_copies) {
            let start = layout.fat_offset + copy * layout.fat_bytes;
            set_fat12(&mut source[start..start + layout.fat_bytes], 4, 0xFFF).unwrap();
        }
        let workspace: WorkspaceManifest =
            serde_json::from_slice(&fs::read(manifest_path).unwrap()).unwrap();
        let original = workspace.archives[0].files[0].clone();
        let mut archive = workspace.archives[0].clone();
        let mut growing = original.clone();
        growing.path = "SECOND.BIN".to_string();
        growing.start_cluster = 4;
        growing.cluster_chain = vec![4];
        growing.size = 1;
        archive.files = vec![growing, original];
        let files = vec![
            PreparedFile {
                data: vec![0; 1500],
                modified: true,
                desired_chain: vec![4, 3],
            },
            PreparedFile {
                data: vec![0; 5],
                modified: true,
                desired_chain: vec![2],
            },
        ];
        rebuild_fat(&mut source, &layout, &archive, &files).unwrap();
        let fat = first_fat(&source, &layout);
        assert_eq!(fat12_next(fat, 4).unwrap(), 3);
        assert!((0xFF8..=0xFFF).contains(&fat12_next(fat, 3).unwrap()));
        assert!((0xFF8..=0xFFF).contains(&fat12_next(fat, 2).unwrap()));
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn rejects_insufficient_space_without_output() {
        let (base, source, manifest, _) = setup("no-space");
        let files = base.join("arbitrary files");
        fs::write(files.join("HELLO.TXT"), vec![0xA5; SECTOR * SECTORS]).unwrap();
        let output = base.join("out.fdi");
        let error = pack_image(&source, &manifest, &files, &output, false).unwrap_err();
        assert!(error.contains("磁盘空间不足"));
        assert!(!output.exists());
        fs::remove_dir_all(base).unwrap();
    }
}
