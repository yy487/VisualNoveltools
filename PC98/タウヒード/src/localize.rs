use crate::ag00::{self, Ag00Document};
use crate::font::{self, EncodingPlan, EncodingPlanEntry};
use crate::script::{self, TextDocument};
use crate::{
    commit_staging, create_unique_sibling, parse_nfd, rebuild_nfd, resolve_inputs,
    safe_input_label, sha256_hex, DiskFile, ParsedImage,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const WORKSPACE_FORMAT: &str = "tauhido-localization-workspace-v1";
const REBUILD_FORMAT: &str = "tauhido-localization-rebuild-v1";

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LocalizationManifest {
    #[serde(rename = "_format")]
    format: String,
    images: Vec<LocalizationImage>,
    documents: Vec<LocalizationDocument>,
    summary: LocalizationSummary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LocalizationImage {
    source_file: String,
    source_sha256: String,
    extract_dir: String,
    members: Vec<LocalizationMember>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LocalizationMember {
    name: String,
    path: String,
    sha256: String,
    size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LocalizationDocument {
    kind: String,
    source_image: String,
    source_member: String,
    json_file: String,
    entries: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LocalizationSummary {
    images: usize,
    extracted_files: usize,
    scenario_files: usize,
    entries: usize,
}

#[derive(Clone, Debug, Serialize)]
struct RebuildManifest {
    #[serde(rename = "_format")]
    format: String,
    images: Vec<RebuiltImage>,
    font: RebuiltFont,
    summary: RebuildSummary,
}

#[derive(Clone, Debug, Serialize)]
struct RebuiltImage {
    source_file: String,
    source_sha256: String,
    output_file: String,
    output_sha256: String,
    replaced_members: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct RebuiltFont {
    output_file: String,
    sha256: String,
    face: String,
    redrawn_slots: usize,
    mappings: Vec<EncodingPlanEntry>,
}

#[derive(Clone, Debug, Serialize)]
struct RebuildSummary {
    images: usize,
    documents: usize,
    entries: usize,
    changed_entries: usize,
}

#[derive(Clone, Debug)]
pub struct LocalizationExtractReport {
    pub images: usize,
    pub extracted_files: usize,
    pub documents: usize,
    pub entries: usize,
    pub output_root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct LocalizationPackReport {
    pub images: usize,
    pub documents: usize,
    pub entries: usize,
    pub changed_entries: usize,
    pub redrawn_slots: usize,
    pub output_root: PathBuf,
}

enum LoadedDocument {
    Scenario(TextDocument),
    Ag00(Ag00Document),
}

impl LoadedDocument {
    fn messages(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        match self {
            Self::Scenario(document) => Box::new(script::messages(document)),
            Self::Ag00(document) => Box::new(ag00::messages(document)),
        }
    }

    fn entries(&self) -> usize {
        match self {
            Self::Scenario(document) => document.entries.len(),
            Self::Ag00(document) => document.entries.len(),
        }
    }
}

pub fn extract_localization(
    inputs: &[PathBuf],
    output_root: &Path,
    overwrite: bool,
) -> Result<LocalizationExtractReport> {
    let paths = resolve_inputs(inputs)?;
    crate::validate_output_does_not_contain_inputs(&paths, output_root)?;
    validate_existing(output_root, overwrite, WORKSPACE_FORMAT, "workspace.json")?;

    let mut images = Vec::with_capacity(paths.len());
    for (index, path) in paths.iter().enumerate() {
        let source =
            fs::read(path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
        let name = path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .ok_or_else(|| format!("输入没有文件名: {}", path.display()))?;
        let parsed =
            parse_nfd(source, name).map_err(|error| format!("{}: {error}", path.display()))?;
        images.push((format!("{index:02}_{}", safe_input_label(path)), parsed));
    }
    let targets = find_targets(&images)?;
    let staging = create_unique_sibling(output_root, "localize-staging")?;
    let result = write_extract_workspace(&images, &targets, &staging);
    let manifest = match result {
        Ok(manifest) => manifest,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    };
    if let Err(error) = commit_staging(&staging, output_root, overwrite) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    Ok(LocalizationExtractReport {
        images: manifest.summary.images,
        extracted_files: manifest.summary.extracted_files,
        documents: manifest.documents.len(),
        entries: manifest.summary.entries,
        output_root: output_root.to_path_buf(),
    })
}

pub fn pack_localization(
    inputs: &[PathBuf],
    workspace_root: &Path,
    output_root: &Path,
    overwrite: bool,
) -> Result<LocalizationPackReport> {
    let manifest: LocalizationManifest = read_json(&workspace_root.join("workspace.json"))?;
    if manifest.format != WORKSPACE_FORMAT {
        return Err("workspace.json 不是 Tauhido 本地化工作区".to_string());
    }
    let paths = resolve_inputs(inputs)?;
    crate::validate_output_does_not_contain_inputs(&paths, output_root)?;
    validate_existing(
        output_root,
        overwrite,
        REBUILD_FORMAT,
        "rebuild_manifest.json",
    )?;

    let mut parsed_sources = Vec::with_capacity(paths.len());
    for path in &paths {
        let source =
            fs::read(path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
        let name = path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .ok_or_else(|| format!("输入没有文件名: {}", path.display()))?;
        parsed_sources.push(parse_nfd(source, name)?);
    }
    let ordered_sources = match_manifest_images(&manifest, parsed_sources)?;

    let mut loaded = Vec::with_capacity(manifest.documents.len());
    for reference in &manifest.documents {
        let path = workspace_root.join(relative_path(&reference.json_file)?);
        let document = match reference.kind.as_str() {
            "scenario" => LoadedDocument::Scenario(read_json(&path)?),
            "ag00" => LoadedDocument::Ag00(read_json(&path)?),
            other => return Err(format!("不支持的文档种类 {other:?}")),
        };
        if document.entries() != reference.entries {
            return Err(format!(
                "{}: JSON 条目数与 workspace.json 不一致",
                path.display()
            ));
        }
        loaded.push(document);
    }
    let plan = EncodingPlan::build(loaded.iter().flat_map(|document| document.messages()))?;
    let requests = plan.requests();
    let font_build = font::prepare_font(&requests, &BTreeSet::new())?;

    let mut replacements_by_image: Vec<HashMap<String, Vec<u8>>> =
        vec![HashMap::new(); ordered_sources.len()];
    let mut changed_entries = 0usize;
    for (reference, document) in manifest.documents.iter().zip(&loaded) {
        let image_index = manifest
            .images
            .iter()
            .position(|image| image.source_file == reference.source_image)
            .ok_or_else(|| format!("文档引用了未知镜像 {}", reference.source_image))?;
        let source = &ordered_sources[image_index];
        let member = find_member(source, &reference.source_member)?;
        let (bytes, changed) = match document {
            LoadedDocument::Scenario(document) => {
                script::rebuild_document(&member.data, &member.name, document, &plan)?
            }
            LoadedDocument::Ag00(document) => {
                ag00::rebuild(&member.data, &member.name, document, &plan)?
            }
        };
        changed_entries += changed;
        if replacements_by_image[image_index]
            .insert(member.name.clone(), bytes)
            .is_some()
        {
            return Err(format!("成员 {} 被多个文档重复替换", member.name));
        }
    }

    let mut rebuilt_images = Vec::with_capacity(ordered_sources.len());
    for (source, replacements) in ordered_sources.iter().zip(&replacements_by_image) {
        rebuilt_images.push(rebuild_nfd(source, replacements)?);
    }

    let staging = create_unique_sibling(output_root, "pack-staging")?;
    let write_result = (|| -> Result<RebuildManifest> {
        let mut image_results = Vec::with_capacity(ordered_sources.len());
        let mut output_names = HashSet::new();
        for ((source, replacements), bytes) in ordered_sources
            .iter()
            .zip(&replacements_by_image)
            .zip(&rebuilt_images)
        {
            if !output_names.insert(source.source_name.to_lowercase()) {
                return Err(format!("输出镜像重名: {}", source.source_name));
            }
            fs::write(staging.join(&source.source_name), bytes)
                .map_err(|error| format!("写入 {} 失败: {error}", source.source_name))?;
            let mut replaced_members = replacements.keys().cloned().collect::<Vec<_>>();
            replaced_members.sort_by_key(|name| name.to_lowercase());
            image_results.push(RebuiltImage {
                source_file: source.source_name.clone(),
                source_sha256: sha256_hex(&source.source),
                output_file: source.source_name.clone(),
                output_sha256: sha256_hex(bytes),
                replaced_members,
            });
        }
        fs::write(staging.join("font.tmp"), &font_build.bytes)
            .map_err(|error| format!("写入 font.tmp 失败: {error}"))?;
        let rebuild = RebuildManifest {
            format: REBUILD_FORMAT.to_string(),
            images: image_results,
            font: RebuiltFont {
                output_file: "font.tmp".to_string(),
                sha256: sha256_hex(&font_build.bytes),
                face: font::FONT_FACE.to_string(),
                redrawn_slots: font_build.patched_glyphs,
                mappings: plan.manifest_entries()?,
            },
            summary: RebuildSummary {
                images: ordered_sources.len(),
                documents: loaded.len(),
                entries: loaded.iter().map(LoadedDocument::entries).sum(),
                changed_entries,
            },
        };
        write_json(&staging.join("rebuild_manifest.json"), &rebuild)?;
        Ok(rebuild)
    })();
    let rebuild = match write_result {
        Ok(rebuild) => rebuild,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    };
    if let Err(error) = commit_staging(&staging, output_root, overwrite) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    Ok(LocalizationPackReport {
        images: rebuild.summary.images,
        documents: rebuild.summary.documents,
        entries: rebuild.summary.entries,
        changed_entries: rebuild.summary.changed_entries,
        redrawn_slots: rebuild.font.redrawn_slots,
        output_root: output_root.to_path_buf(),
    })
}

fn write_extract_workspace(
    images: &[(String, ParsedImage)],
    targets: &[(usize, usize, &'static str)],
    staging: &Path,
) -> Result<LocalizationManifest> {
    let extract_root = staging.join("extract");
    let translation_root = staging.join("translation_json");
    let profile_root = staging.join("profile");
    let analysis_root = staging.join("analysis");
    fs::create_dir_all(&extract_root).map_err(|error| format!("创建 extract 失败: {error}"))?;
    fs::create_dir_all(&translation_root)
        .map_err(|error| format!("创建 translation_json 失败: {error}"))?;
    fs::create_dir_all(&profile_root).map_err(|error| format!("创建 profile 失败: {error}"))?;
    fs::create_dir_all(&analysis_root).map_err(|error| format!("创建 analysis 失败: {error}"))?;

    let mut image_manifests = Vec::with_capacity(images.len());
    let mut extracted_files = 0usize;
    for (directory, image) in images {
        let image_root = extract_root.join(directory);
        fs::create_dir(&image_root)
            .map_err(|error| format!("创建 {} 失败: {error}", image_root.display()))?;
        let mut members = Vec::with_capacity(image.files.len());
        for file in &image.files {
            if file.name.eq_ignore_ascii_case("NACT8S") {
                continue;
            }
            fs::write(image_root.join(&file.name), &file.data)
                .map_err(|error| format!("写入 {} 失败: {error}", file.name))?;
            members.push(LocalizationMember {
                name: file.name.clone(),
                path: format!("extract/{directory}/{}", file.name),
                sha256: sha256_hex(&file.data),
                size: file.data.len() as u64,
            });
            extracted_files += 1;
        }
        image_manifests.push(LocalizationImage {
            source_file: image.source_name.clone(),
            source_sha256: sha256_hex(&image.source),
            extract_dir: format!("extract/{directory}"),
            members,
        });
    }

    let mut documents = Vec::new();
    let mut entry_total = 0usize;
    let mut scenario_files = 0usize;
    for (image_index, file_index, kind) in targets {
        let image = &images[*image_index].1;
        let file = &image.files[*file_index];
        let json_name = format!("{}.json", file.name);
        let json_path = translation_root.join(&json_name);
        let entries = match *kind {
            "scenario" => {
                let document = script::extract_document(&file.data, &file.name)?;
                let count = document.entries.len();
                write_json(&json_path, &document)?;
                scenario_files += 1;
                count
            }
            "ag00" => {
                let document = ag00::extract_document(&file.data, &file.name)?;
                let count = document.entries.len();
                write_json(&json_path, &document)?;
                count
            }
            _ => return Err("内部文档种类无效".to_string()),
        };
        entry_total += entries;
        documents.push(LocalizationDocument {
            kind: (*kind).to_string(),
            source_image: image.source_name.clone(),
            source_member: file.name.clone(),
            json_file: format!("translation_json/{json_name}"),
            entries,
        });
    }
    documents.sort_by_key(|document| document.source_member.to_lowercase());

    let profile = serde_json::json!({
        "_format": "tauhido-project-profile-v1",
        "encoding": "CP932 carrier slots; DISK uses hybrid compressed kana, AG00 stores 7-bit JIS inside ESC K / ESC H",
        "editable_field": "message",
        "immutable_field": "scr_msg",
        "automatic_wrap": false,
        "maximum_columns": 40,
        "controls": {
            "structural": ["A", "B", "F", "G", "L", "M", "P", "Q", "R", "S", "U", "X", "Y", "Z", "!", "&", "@", "$", "[", ":", "]", "{", "}"],
            "R": "new line and reset X to 1",
            "B": "new line and reset X to 9",
            "A": "wait/page prompt"
        },
        "font": {
            "base": "embedded font.tmp",
            "mapping": "embedded subs_cn_jp.json with deterministic collision fallback",
            "redraw_policy": "redraw every carrier slot used by final DISK-A, DISK-B and AG00 text",
            "ignored_program": "NACT8S"
        }
    });
    write_json(&profile_root.join("project.json"), &profile)?;
    let format_note = "# Tauhido text formats\n\nDISK-A and DISK-B are parsed as 256-byte records with a record pointer table, a page map, per-script command tables, bytecode, local pointers, choices, and text spans. R and B are structural line controls; the engine does not wrap horizontally.\n\nAG00 is a separate table: an ASCII count header followed by verb/object lines. Normal lines store 7-bit JIS pairs between ESC K and ESC H; the first object `*` remains structural.\n\nPacking plans one global carrier slot per final character, rebuilds every used slot in `font.tmp`, updates script-local and file-level pointers, then rewrites the N88 directory/FAT/member sectors in each NFD image. NACT8S is deliberately excluded.\n";
    fs::write(analysis_root.join("FORMAT.md"), format_note.as_bytes())
        .map_err(|error| format!("写入 FORMAT.md 失败: {error}"))?;

    let manifest = LocalizationManifest {
        format: WORKSPACE_FORMAT.to_string(),
        images: image_manifests,
        documents,
        summary: LocalizationSummary {
            images: images.len(),
            extracted_files,
            scenario_files,
            entries: entry_total,
        },
    };
    write_json(&staging.join("workspace.json"), &manifest)?;
    Ok(manifest)
}

fn find_targets(images: &[(String, ParsedImage)]) -> Result<Vec<(usize, usize, &'static str)>> {
    let wanted = [
        ("AG00", "ag00"),
        ("DISK-A", "scenario"),
        ("DISK-B", "scenario"),
    ];
    let mut targets = Vec::with_capacity(wanted.len());
    for (name, kind) in wanted {
        let found = images
            .iter()
            .enumerate()
            .flat_map(|(image_index, (_, image))| {
                image
                    .files
                    .iter()
                    .enumerate()
                    .filter_map(move |(file_index, file)| {
                        file.name.eq_ignore_ascii_case(name).then_some((
                            image_index,
                            file_index,
                            kind,
                        ))
                    })
            })
            .collect::<Vec<_>>();
        if found.len() != 1 {
            return Err(format!(
                "应恰好找到一个 {name}，实际找到 {} 个",
                found.len()
            ));
        }
        targets.push(found[0]);
    }
    Ok(targets)
}

fn match_manifest_images(
    manifest: &LocalizationManifest,
    mut sources: Vec<ParsedImage>,
) -> Result<Vec<ParsedImage>> {
    if sources.len() != manifest.images.len() {
        return Err(format!(
            "源镜像数量与工作区不一致: 当前 {}，工作区 {}",
            sources.len(),
            manifest.images.len()
        ));
    }
    let mut ordered = Vec::with_capacity(sources.len());
    for expected in &manifest.images {
        let candidates = sources
            .iter()
            .enumerate()
            .filter(|(_, source)| {
                source
                    .source_name
                    .eq_ignore_ascii_case(&expected.source_file)
                    && sha256_hex(&source.source) == expected.source_sha256
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            return Err(format!(
                "{}: 按文件名和 SHA-256 应匹配一个源镜像，实际 {} 个",
                expected.source_file,
                candidates.len()
            ));
        }
        ordered.push(sources.remove(candidates[0]));
    }
    Ok(ordered)
}

fn find_member<'a>(image: &'a ParsedImage, name: &str) -> Result<&'a DiskFile> {
    image
        .files
        .iter()
        .find(|file| file.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| format!("{} 中没有成员 {name}", image.source_name))
}

fn validate_existing(root: &Path, overwrite: bool, format: &str, marker: &str) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    if !root.is_dir() {
        return Err(format!("输出已存在且不是目录: {}", root.display()));
    }
    if !overwrite {
        return Err(format!(
            "输出目录已存在；需要显式 --overwrite: {}",
            root.display()
        ));
    }
    let mut entries = fs::read_dir(root).map_err(|error| format!("读取输出目录失败: {error}"))?;
    if entries.next().is_none() {
        return Ok(());
    }
    let value: serde_json::Value = read_json(&root.join(marker))?;
    if value.get("_format").and_then(|item| item.as_str()) != Some(format) {
        return Err(format!("拒绝覆盖其他格式工作区: {}", root.display()));
    }
    Ok(())
}

fn relative_path(value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(format!("工作区清单含不安全相对路径: {value:?}"));
    }
    Ok(path.to_path_buf())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("{}: JSON 无效: {error}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("序列化 {} 失败: {error}", path.display()))?;
    json.push('\n');
    fs::write(path, json.as_bytes())
        .map_err(|error| format!("写入 {} 失败: {error}", path.display()))
}
