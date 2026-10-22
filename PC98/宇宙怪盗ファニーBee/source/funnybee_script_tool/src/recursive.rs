use crate::{
    analyze_script, export_plain_text, extract_translation, extract_translation_with_gaiji,
    import_plain_text, inject_translation, inject_translation_localized, prepare_font,
    FontPatchRequest, GaijiTable, GlyphTable, InjectOptions, InjectReport, SubstitutionMap,
    TranslationDocument, FUNNYBEE_GLYPH_TABLE_SHA256,
};
use funnybee_dat_unpack::{parse_archive, rebuild_archive, SECTOR_SIZE};
use pc98_fdi_unpack::{extract_fdi_files, rebuild_fdi};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, String>;

const WORKSPACE_FORMAT: &str = "funnybee-recursive-localization-workspace-v1";
const MAX_CONTAINER_DEPTH: usize = 8;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct Locator {
    root: String,
    layers: Vec<LocatorLayer>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum LocatorLayer {
    FdiFile { path: String },
    DatMember { index: usize },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ContainerKind {
    FdiFat12,
    DriDat,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ContainerRecord {
    id: usize,
    kind: ContainerKind,
    locator: Locator,
    #[serde(default)]
    source_file: String,
    source_sha256: String,
    extract_dir: String,
    children: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ScriptRecord {
    id: usize,
    locator: Locator,
    source_file: String,
    source_sha256: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    json_file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    txt_file: Option<String>,
    entries: usize,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum TranslationOutputFormat {
    #[default]
    Json,
    PlainTxt,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AdvRecord {
    locator: Option<Locator>,
    source_sha256: String,
    glyph_table_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WorkspaceManifest {
    _format: String,
    tool_version: String,
    source_kind: SourceKind,
    source_name: String,
    #[serde(default)]
    translation_output: TranslationOutputFormat,
    adv: Option<AdvRecord>,
    containers: Vec<ContainerRecord>,
    scripts: Vec<ScriptRecord>,
    warnings: Vec<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SourceKind {
    File,
    Directory,
}

#[derive(Debug)]
struct Node {
    locator: Locator,
    source_file: String,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct RecursiveExtractReport {
    pub scanned_root_files: usize,
    pub containers: usize,
    pub fdi_images: usize,
    pub dat_archives: usize,
    pub dat_outputs: Vec<String>,
    pub scripts: usize,
    pub entries: usize,
    pub warnings: Vec<String>,
    pub output: PathBuf,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RecursivePackOptions {
    pub max_columns: Option<usize>,
    pub overwrite: bool,
}

#[derive(Debug, Clone)]
pub struct RecursivePackReport {
    pub containers: usize,
    pub scripts: usize,
    pub changed_scripts: usize,
    pub changed_dat_archives: usize,
    pub changed_fdi_images: usize,
    pub font_glyphs: usize,
    pub font_output: Option<PathBuf>,
    pub output: PathBuf,
}

pub fn recursive_extract(
    input: &Path,
    adv_path: Option<&Path>,
    output: &Path,
    overwrite: bool,
) -> Result<RecursiveExtractReport> {
    recursive_extract_with_format(
        input,
        adv_path,
        output,
        overwrite,
        TranslationOutputFormat::Json,
    )
}

pub fn recursive_extract_txt(
    input: &Path,
    adv_path: Option<&Path>,
    output: &Path,
    overwrite: bool,
) -> Result<RecursiveExtractReport> {
    recursive_extract_with_format(
        input,
        adv_path,
        output,
        overwrite,
        TranslationOutputFormat::PlainTxt,
    )
}

fn recursive_extract_with_format(
    input: &Path,
    adv_path: Option<&Path>,
    output: &Path,
    overwrite: bool,
    translation_output: TranslationOutputFormat,
) -> Result<RecursiveExtractReport> {
    validate_input_output(input, output)?;
    validate_workspace_output(output, overwrite)?;
    let (source_kind, source_name, roots) = read_root_nodes(input)?;
    let scanned_root_files = roots.len();
    let mut queue = VecDeque::from(roots);
    let mut containers = Vec::new();
    let mut leaves = Vec::new();
    let mut prepared_files = BTreeMap::<String, Vec<u8>>::new();
    let mut automatic_adv = Vec::<Node>::new();
    let mut next_container_id = 1usize;

    while let Some(node) = queue.pop_front() {
        if node.locator.layers.len() > MAX_CONTAINER_DEPTH {
            return Err(format!(
                "容器递归超过 {MAX_CONTAINER_DEPTH} 层: {}",
                describe_locator(&node.locator)
            ));
        }
        if let Ok(glyphs) = GlyphTable::from_adv(&node.bytes) {
            if glyphs.raw_sha256 == FUNNYBEE_GLYPH_TABLE_SHA256 {
                automatic_adv.push(Node {
                    locator: node.locator.clone(),
                    source_file: node.source_file.clone(),
                    bytes: node.bytes.clone(),
                });
            }
        }

        if probable_fdi(&node.bytes) {
            let Ok(files) = extract_fdi_files(&node.bytes, &node.source_file) else {
                leaves.push(node);
                continue;
            };
            let id = next_container_id;
            next_container_id += 1;
            let extract_dir = format!("extract/fdi_{id:04}");
            for file in files {
                let output_path = manifest_join(&extract_dir, &file.path)?;
                insert_prepared(&mut prepared_files, output_path, file.data.clone())?;
                let mut locator = node.locator.clone();
                locator.layers.push(LocatorLayer::FdiFile {
                    path: file.path.clone(),
                });
                queue.push_back(Node {
                    locator,
                    source_file: leaf_name(&file.path)?.to_string(),
                    bytes: file.data,
                });
            }
            containers.push(ContainerRecord {
                id,
                kind: ContainerKind::FdiFat12,
                locator: node.locator,
                source_file: node.source_file,
                source_sha256: sha256_hex(&node.bytes),
                extract_dir,
                children: prepared_child_count(&prepared_files, id, ContainerKind::FdiFat12),
            });
            continue;
        }

        if probable_dat(&node.bytes) {
            let Ok(parsed) = parse_archive(&node.bytes) else {
                leaves.push(node);
                continue;
            };
            let id = next_container_id;
            next_container_id += 1;
            let extract_dir = dat_extract_dir(id, &node.source_file);
            insert_prepared(
                &mut prepared_files,
                format!("{extract_dir}/header.bin"),
                node.bytes[..SECTOR_SIZE].to_vec(),
            )?;
            insert_prepared(
                &mut prepared_files,
                format!("{extract_dir}/routing.bin"),
                node.bytes[parsed.route_table_offset
                    ..parsed.route_table_offset + parsed.route_table_size]
                    .to_vec(),
            )?;
            for member in &parsed.members {
                let member_path = format!("members/member_{:03}.bin", member.local_member);
                let bytes = node.bytes[member.offset..member.offset + member.size].to_vec();
                insert_prepared(
                    &mut prepared_files,
                    format!("{extract_dir}/{member_path}"),
                    bytes.clone(),
                )?;
                let mut locator = node.locator.clone();
                locator.layers.push(LocatorLayer::DatMember {
                    index: member.local_member,
                });
                queue.push_back(Node {
                    locator,
                    source_file: format!("member_{:03}.bin", member.local_member),
                    bytes,
                });
            }
            containers.push(ContainerRecord {
                id,
                kind: ContainerKind::DriDat,
                locator: node.locator,
                source_file: node.source_file,
                source_sha256: sha256_hex(&node.bytes),
                extract_dir,
                children: parsed.members.len(),
            });
            continue;
        }
        leaves.push(node);
    }

    let mut warnings = Vec::new();
    let (glyphs, adv_record) = if let Some(path) = adv_path {
        let bytes =
            fs::read(path).map_err(|error| format!("无法读取 ADV {}: {error}", path.display()))?;
        let glyphs = GlyphTable::from_adv(&bytes)?;
        let record = AdvRecord {
            locator: None,
            source_sha256: sha256_hex(&bytes),
            glyph_table_sha256: glyphs.raw_sha256.clone(),
        };
        (Some(glyphs), Some(record))
    } else if let Some(node) = automatic_adv.first() {
        if automatic_adv.len() > 1 {
            warnings.push(format!(
                "发现 {} 个相同 FunnyBee 字形表候选，采用 {}",
                automatic_adv.len(),
                describe_locator(&node.locator)
            ));
        }
        let glyphs = GlyphTable::from_adv(&node.bytes)?;
        let record = AdvRecord {
            locator: Some(node.locator.clone()),
            source_sha256: sha256_hex(&node.bytes),
            glyph_table_sha256: glyphs.raw_sha256.clone(),
        };
        (Some(glyphs), Some(record))
    } else {
        warnings.push(
            "未提供且未从输入内容发现 FunnyBee ADV 字形表；已解包容器，但跳过脚本文本".to_string(),
        );
        (None, None)
    };

    let mut scripts = Vec::new();
    let gaiji = if translation_output == TranslationOutputFormat::PlainTxt {
        Some(GaijiTable::builtin()?)
    } else {
        None
    };
    if let Some(glyphs) = &glyphs {
        for leaf in leaves {
            let Ok(analysis) = analyze_script(&leaf.bytes, glyphs, &leaf.source_file) else {
                continue;
            };
            let Some(terminator_offset) = analysis.terminator_offset else {
                continue;
            };
            if usize::from(analysis.initial_word) > terminator_offset {
                continue;
            }
            let document = match &gaiji {
                Some(gaiji) => {
                    extract_translation_with_gaiji(&leaf.bytes, glyphs, &leaf.source_file, gaiji)?
                }
                None => extract_translation(&leaf.bytes, glyphs, &leaf.source_file)?,
            };
            let id = scripts.len() + 1;
            let (json_file, txt_file, translation_file, translation_bytes) =
                match translation_output {
                    TranslationOutputFormat::Json => {
                        let path = format!("translation_json/script_{id:05}.json");
                        (path.clone(), None, path, json_bytes(&document)?)
                    }
                    TranslationOutputFormat::PlainTxt => {
                        let path = format!(
                            "translation_txt/script_{id:05}_{}.txt",
                            safe_output_label(&leaf.source_file)
                        );
                        (
                            String::new(),
                            Some(path.clone()),
                            path,
                            export_plain_text(&document)?,
                        )
                    }
                };
            insert_prepared(&mut prepared_files, translation_file, translation_bytes)?;
            scripts.push(ScriptRecord {
                id,
                locator: leaf.locator,
                source_file: leaf.source_file,
                source_sha256: analysis.source_sha256,
                json_file,
                txt_file,
                entries: document.entries.len(),
            });
        }
    }

    let manifest = WorkspaceManifest {
        _format: WORKSPACE_FORMAT.to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        source_kind,
        source_name,
        translation_output,
        adv: adv_record,
        containers,
        scripts,
        warnings: warnings.clone(),
    };
    insert_prepared(
        &mut prepared_files,
        "workspace.json".to_string(),
        json_bytes(&manifest)?,
    )?;
    write_workspace(output, overwrite, &prepared_files)?;

    let fdi_images = manifest
        .containers
        .iter()
        .filter(|record| record.kind == ContainerKind::FdiFat12)
        .count();
    let dat_archives = manifest
        .containers
        .iter()
        .filter(|record| record.kind == ContainerKind::DriDat)
        .count();
    let dat_outputs = manifest
        .containers
        .iter()
        .filter(|record| record.kind == ContainerKind::DriDat)
        .map(|record| {
            format!(
                "{} -> {} ({} members)",
                record.source_file, record.extract_dir, record.children
            )
        })
        .collect();
    Ok(RecursiveExtractReport {
        scanned_root_files,
        containers: manifest.containers.len(),
        fdi_images,
        dat_archives,
        dat_outputs,
        scripts: manifest.scripts.len(),
        entries: manifest.scripts.iter().map(|record| record.entries).sum(),
        warnings,
        output: output.to_path_buf(),
    })
}

fn dat_extract_dir(id: usize, source_file: &str) -> String {
    format!("extract/dat_{id:04}_{}", safe_output_label(source_file))
}

fn safe_output_label(value: &str) -> String {
    let mut label = String::new();
    let mut previous_was_separator = false;
    for character in value.chars().take(64) {
        if character.is_alphanumeric() || matches!(character, '-' | '_') {
            label.push(character);
            previous_was_separator = false;
        } else if !previous_was_separator {
            label.push('_');
            previous_was_separator = true;
        }
    }
    let label = label.trim_matches('_');
    if label.is_empty() {
        "unnamed".to_string()
    } else {
        label.to_string()
    }
}

pub fn recursive_pack(
    source: &Path,
    workspace: &Path,
    adv_path: Option<&Path>,
    output: &Path,
    options: RecursivePackOptions,
) -> Result<RecursivePackReport> {
    recursive_pack_with_format(
        source,
        workspace,
        adv_path,
        output,
        options,
        TranslationOutputFormat::Json,
    )
}

pub fn recursive_pack_txt(
    source: &Path,
    workspace: &Path,
    adv_path: Option<&Path>,
    output: &Path,
    options: RecursivePackOptions,
) -> Result<RecursivePackReport> {
    recursive_pack_with_format(
        source,
        workspace,
        adv_path,
        output,
        options,
        TranslationOutputFormat::PlainTxt,
    )
}

fn recursive_pack_with_format(
    source: &Path,
    workspace: &Path,
    adv_path: Option<&Path>,
    output: &Path,
    options: RecursivePackOptions,
    expected_translation_output: TranslationOutputFormat,
) -> Result<RecursivePackReport> {
    validate_input_output(source, output)?;
    let manifest: WorkspaceManifest = read_json(&workspace.join("workspace.json"))?;
    if manifest._format != WORKSPACE_FORMAT {
        return Err(format!("不支持的递归工作区格式: {}", manifest._format));
    }
    if manifest.translation_output != expected_translation_output {
        return Err(format!(
            "工作区翻译格式为 {:?}，当前命令要求 {:?}",
            manifest.translation_output, expected_translation_output
        ));
    }
    validate_source_kind(source, manifest.source_kind)?;
    let glyphs = load_pack_glyphs(source, adv_path, &manifest)?;
    let gaiji = if expected_translation_output == TranslationOutputFormat::PlainTxt {
        Some(GaijiTable::builtin()?)
    } else {
        None
    };
    let substitutions = if expected_translation_output == TranslationOutputFormat::PlainTxt {
        Some(SubstitutionMap::embedded()?)
    } else {
        None
    };

    let mut scripts_by_parent = BTreeMap::<Locator, Vec<&ScriptRecord>>::new();
    let mut root_scripts = Vec::new();
    for script in &manifest.scripts {
        if let Some(parent) = parent_locator(&script.locator) {
            scripts_by_parent.entry(parent).or_default().push(script);
        } else {
            root_scripts.push(script);
        }
    }
    let mut containers_by_parent = BTreeMap::<Locator, Vec<&ContainerRecord>>::new();
    for container in &manifest.containers {
        if let Some(parent) = parent_locator(&container.locator) {
            containers_by_parent
                .entry(parent)
                .or_default()
                .push(container);
        }
    }

    let mut ordered = manifest.containers.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        right
            .locator
            .layers
            .len()
            .cmp(&left.locator.layers.len())
            .then_with(|| right.id.cmp(&left.id))
    });
    let mut rebuilt_nodes = BTreeMap::<Locator, Vec<u8>>::new();
    let mut changed_scripts = 0usize;
    let mut changed_dat_archives = 0usize;
    let mut changed_fdi_images = 0usize;
    let mut font_requests = Vec::<FontPatchRequest>::new();
    let mut literal_characters = BTreeSet::<char>::new();

    for container in ordered {
        let original = resolve_original(source, &container.locator)?;
        verify_sha(
            &original,
            &container.source_sha256,
            &describe_locator(&container.locator),
        )?;
        let rebuilt = match container.kind {
            ContainerKind::DriDat => {
                let parsed = parse_archive(&original)?;
                let root = workspace_join(workspace, &container.extract_dir)?;
                let mut members = Vec::with_capacity(parsed.members.len());
                for member in &parsed.members {
                    members.push(
                        fs::read(
                            root.join(format!("members/member_{:03}.bin", member.local_member)),
                        )
                        .map_err(|error| {
                            format!(
                                "无法读取 DAT 成员 {} #{}: {error}",
                                describe_locator(&container.locator),
                                member.local_member
                            )
                        })?,
                    );
                }
                if let Some(children) = containers_by_parent.get(&container.locator) {
                    for child in children {
                        let index = child_dat_member(&child.locator)?;
                        let payload = rebuilt_nodes.get(&child.locator).ok_or_else(|| {
                            format!(
                                "内部错误：子容器尚未重建: {}",
                                describe_locator(&child.locator)
                            )
                        })?;
                        let slot = members
                            .get_mut(index - 1)
                            .ok_or_else(|| format!("DAT 子容器成员号越界: {index}"))?;
                        *slot = payload.clone();
                    }
                }
                if let Some(scripts) = scripts_by_parent.get(&container.locator) {
                    for script in scripts {
                        let index = script_dat_member(&script.locator)?;
                        let slot = members
                            .get_mut(index - 1)
                            .ok_or_else(|| format!("脚本 DAT 成员号越界: {index}"))?;
                        let document = load_translation_document(
                            workspace,
                            script,
                            slot,
                            &glyphs,
                            expected_translation_output,
                            gaiji.as_ref(),
                        )?;
                        let report = inject_document(
                            slot,
                            &glyphs,
                            &script.source_file,
                            &document,
                            InjectOptions {
                                max_columns: options.max_columns,
                            },
                            gaiji.as_ref(),
                            substitutions.as_ref(),
                        )?;
                        changed_scripts += usize::from(report.changed_entries > 0);
                        font_requests.extend(report.font_requests);
                        literal_characters.extend(report.literal_characters);
                        *slot = report.bytes;
                    }
                }
                let report = rebuild_archive(&original, &members)?;
                changed_dat_archives += usize::from(report.changed_members > 0);
                report.bytes
            }
            ContainerKind::FdiFat12 => {
                let files = extract_fdi_files(&original, &container_filename(&container.locator))?;
                let root = workspace_join(workspace, &container.extract_dir)?;
                let mut replacements = BTreeMap::new();
                for file in &files {
                    let workspace_file = manifest_path_join(&root, &file.path)?;
                    let data = fs::read(&workspace_file).map_err(|error| {
                        format!(
                            "无法读取 FDI 解包文件 {}: {error}",
                            workspace_file.display()
                        )
                    })?;
                    if data != file.data {
                        replacements.insert(file.path.clone(), data);
                    }
                }
                if let Some(children) = containers_by_parent.get(&container.locator) {
                    for child in children {
                        let path = child_fdi_path(&child.locator)?;
                        let payload = rebuilt_nodes.get(&child.locator).ok_or_else(|| {
                            format!(
                                "内部错误：子容器尚未重建: {}",
                                describe_locator(&child.locator)
                            )
                        })?;
                        replacements.insert(path, payload.clone());
                    }
                }
                if let Some(scripts) = scripts_by_parent.get(&container.locator) {
                    for script in scripts {
                        let path = script_fdi_path(&script.locator)?;
                        let current = replacements
                            .get(&path)
                            .cloned()
                            .or_else(|| {
                                files
                                    .iter()
                                    .find(|file| file.path == path)
                                    .map(|file| file.data.clone())
                            })
                            .ok_or_else(|| format!("FDI 中找不到脚本文件: {path}"))?;
                        let document = load_translation_document(
                            workspace,
                            script,
                            &current,
                            &glyphs,
                            expected_translation_output,
                            gaiji.as_ref(),
                        )?;
                        let report = inject_document(
                            &current,
                            &glyphs,
                            &script.source_file,
                            &document,
                            InjectOptions {
                                max_columns: options.max_columns,
                            },
                            gaiji.as_ref(),
                            substitutions.as_ref(),
                        )?;
                        changed_scripts += usize::from(report.changed_entries > 0);
                        font_requests.extend(report.font_requests);
                        literal_characters.extend(report.literal_characters);
                        replacements.insert(path, report.bytes);
                    }
                }
                let report = rebuild_fdi(
                    &original,
                    &container_filename(&container.locator),
                    &replacements,
                )?;
                changed_fdi_images += usize::from(report.changed_files > 0);
                report.bytes
            }
        };
        rebuilt_nodes.insert(container.locator.clone(), rebuilt);
    }

    let mut root_replacements = BTreeMap::<String, Vec<u8>>::new();
    for container in &manifest.containers {
        if container.locator.layers.is_empty() {
            let bytes = rebuilt_nodes
                .get(&container.locator)
                .ok_or_else(|| format!("根容器未重建: {}", describe_locator(&container.locator)))?;
            root_replacements.insert(container.locator.root.clone(), bytes.clone());
        }
    }
    for script in root_scripts {
        let current = resolve_original(source, &script.locator)?;
        let document = load_translation_document(
            workspace,
            script,
            &current,
            &glyphs,
            expected_translation_output,
            gaiji.as_ref(),
        )?;
        let report = inject_document(
            &current,
            &glyphs,
            &script.source_file,
            &document,
            InjectOptions {
                max_columns: options.max_columns,
            },
            gaiji.as_ref(),
            substitutions.as_ref(),
        )?;
        changed_scripts += usize::from(report.changed_entries > 0);
        font_requests.extend(report.font_requests);
        literal_characters.extend(report.literal_characters);
        root_replacements.insert(script.locator.root.clone(), report.bytes);
    }

    let font_build = if expected_translation_output == TranslationOutputFormat::PlainTxt {
        Some(prepare_font(&font_requests, &literal_characters)?)
    } else {
        None
    };
    let font_glyphs = font_build
        .as_ref()
        .map(|build| build.patched_glyphs)
        .unwrap_or(0);
    let font_output = write_packed_source(
        source,
        output,
        manifest.source_kind,
        &root_replacements,
        options.overwrite,
        font_build.as_ref().map(|build| build.bytes.as_slice()),
    )?;
    Ok(RecursivePackReport {
        containers: manifest.containers.len(),
        scripts: manifest.scripts.len(),
        changed_scripts,
        changed_dat_archives,
        changed_fdi_images,
        font_glyphs,
        font_output,
        output: output.to_path_buf(),
    })
}

fn load_translation_document(
    workspace: &Path,
    script: &ScriptRecord,
    source: &[u8],
    glyphs: &GlyphTable,
    format: TranslationOutputFormat,
    gaiji: Option<&GaijiTable>,
) -> Result<TranslationDocument> {
    verify_sha(
        source,
        &script.source_sha256,
        &format!("脚本 {}", script.source_file),
    )?;
    match format {
        TranslationOutputFormat::Json => read_json(&workspace_join(workspace, &script.json_file)?),
        TranslationOutputFormat::PlainTxt => {
            let txt_file = script
                .txt_file
                .as_deref()
                .ok_or_else(|| format!("脚本 {} 缺少 TXT 路径记录", script.id))?;
            let path = workspace_join(workspace, txt_file)?;
            let bytes = fs::read(&path)
                .map_err(|error| format!("无法读取 TXT {}: {error}", path.display()))?;
            let gaiji = gaiji.ok_or_else(|| "TXT 工作区缺少 GAIJI 映射".to_string())?;
            let mut document =
                extract_translation_with_gaiji(source, glyphs, &script.source_file, gaiji)?;
            import_plain_text(&mut document, &bytes)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            Ok(document)
        }
    }
}

fn inject_document(
    source: &[u8],
    glyphs: &GlyphTable,
    source_file: &str,
    document: &TranslationDocument,
    options: InjectOptions,
    gaiji: Option<&GaijiTable>,
    substitutions: Option<&SubstitutionMap>,
) -> Result<InjectReport> {
    match (gaiji, substitutions) {
        (Some(gaiji), Some(substitutions)) => inject_translation_localized(
            source,
            glyphs,
            source_file,
            document,
            options,
            gaiji,
            substitutions,
        ),
        (None, None) => inject_translation(source, glyphs, source_file, document, options),
        _ => Err("内部错误：GAIJI 与中文字库映射模式不一致".to_string()),
    }
}

fn load_pack_glyphs(
    source: &Path,
    adv_path: Option<&Path>,
    manifest: &WorkspaceManifest,
) -> Result<GlyphTable> {
    let bytes =
        if let Some(path) = adv_path {
            fs::read(path).map_err(|error| format!("无法读取 ADV {}: {error}", path.display()))?
        } else {
            let record = manifest.adv.as_ref().ok_or_else(|| {
                "工作区没有可回溯的 ADV；pack-recursive 需要显式 --adv".to_string()
            })?;
            let locator = record.locator.as_ref().ok_or_else(|| {
                "提取时使用了外部 ADV；pack-recursive 需要再次显式 --adv".to_string()
            })?;
            let bytes = resolve_original(source, locator)?;
            verify_sha(&bytes, &record.source_sha256, "ADV")?;
            bytes
        };
    let glyphs = GlyphTable::from_adv(&bytes)?;
    if let Some(record) = &manifest.adv {
        if glyphs.raw_sha256 != record.glyph_table_sha256 {
            return Err("当前 ADV 字形表与提取工作区不一致".to_string());
        }
    }
    Ok(glyphs)
}

fn read_root_nodes(input: &Path) -> Result<(SourceKind, String, Vec<Node>)> {
    if input.is_file() {
        let name = input
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("输入文件名无法表示为 Unicode: {}", input.display()))?
            .to_string();
        let bytes =
            fs::read(input).map_err(|error| format!("无法读取 {}: {error}", input.display()))?;
        return Ok((
            SourceKind::File,
            name.clone(),
            vec![Node {
                locator: Locator {
                    root: name.clone(),
                    layers: Vec::new(),
                },
                source_file: name.clone(),
                bytes,
            }],
        ));
    }
    if !input.is_dir() {
        return Err(format!("输入既不是文件也不是目录: {}", input.display()));
    }
    let mut files = Vec::new();
    collect_files(input, input, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let nodes = files
        .into_iter()
        .map(|(relative, path)| {
            let source_file = leaf_name(&relative)?.to_string();
            let bytes =
                fs::read(&path).map_err(|error| format!("无法读取 {}: {error}", path.display()))?;
            Ok(Node {
                locator: Locator {
                    root: relative,
                    layers: Vec::new(),
                },
                source_file,
                bytes,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let source_name = input
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("input")
        .to_string();
    Ok((SourceKind::Directory, source_name, nodes))
}

fn collect_files(root: &Path, current: &Path, output: &mut Vec<(String, PathBuf)>) -> Result<()> {
    let mut entries = fs::read_dir(current)
        .map_err(|error| format!("无法读取目录 {}: {error}", current.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("无法枚举目录 {}: {error}", current.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry
            .file_type()
            .map_err(|error| format!("无法读取文件类型 {}: {error}", entry.path().display()))?;
        if file_type.is_symlink() {
            return Err(format!(
                "递归输入不接受符号链接: {}",
                entry.path().display()
            ));
        }
        if file_type.is_dir() {
            collect_files(root, &entry.path(), output)?;
        } else if file_type.is_file() {
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|_| "计算相对路径失败".to_string())?
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            output.push((relative, entry.path()));
        }
    }
    Ok(())
}

fn resolve_original(source: &Path, locator: &Locator) -> Result<Vec<u8>> {
    let root_path = if source.is_file() {
        let name = source
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("源文件名无法表示为 Unicode: {}", source.display()))?;
        if name != locator.root {
            return Err(format!(
                "工作区根文件 {:?} 与当前源文件 {:?} 不一致",
                locator.root, name
            ));
        }
        source.to_path_buf()
    } else {
        manifest_path_join(source, &locator.root)?
    };
    let mut bytes = fs::read(&root_path)
        .map_err(|error| format!("无法读取源文件 {}: {error}", root_path.display()))?;
    let mut label = locator.root.clone();
    for layer in &locator.layers {
        match layer {
            LocatorLayer::FdiFile { path } => {
                let files = extract_fdi_files(&bytes, &label)?;
                let file = files
                    .into_iter()
                    .find(|file| file.path == *path)
                    .ok_or_else(|| format!("FDI {label} 中缺少文件 {path}"))?;
                label = path.clone();
                bytes = file.data;
            }
            LocatorLayer::DatMember { index } => {
                let parsed = parse_archive(&bytes)?;
                let member = parsed
                    .members
                    .get(index - 1)
                    .filter(|member| member.local_member == *index)
                    .ok_or_else(|| format!("DAT {label} 中缺少成员 #{index}"))?;
                bytes = bytes[member.offset..member.offset + member.size].to_vec();
                label = format!("member_{index:03}.bin");
            }
        }
    }
    Ok(bytes)
}

fn probable_fdi(bytes: &[u8]) -> bool {
    if bytes.len() < 32 {
        return false;
    }
    let data_offset = u32_at(bytes, 8).unwrap_or(0) as usize;
    let data_size = u32_at(bytes, 12).unwrap_or(0) as usize;
    let geometry = [16usize, 20, 24, 28]
        .into_iter()
        .try_fold(1u64, |value, offset| {
            value.checked_mul(u64::from(u32_at(bytes, offset)?))
        });
    data_offset >= 32
        && data_offset.checked_add(data_size) == Some(bytes.len())
        && geometry == Some(data_size as u64)
}

fn probable_dat(bytes: &[u8]) -> bool {
    bytes.len() >= SECTOR_SIZE * 3
        && bytes.len().is_multiple_of(SECTOR_SIZE)
        && bytes.get(..2) == Some(&[2, 0])
}

fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    let raw = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn parent_locator(locator: &Locator) -> Option<Locator> {
    if locator.layers.is_empty() {
        None
    } else {
        let mut parent = locator.clone();
        parent.layers.pop();
        Some(parent)
    }
}

fn child_dat_member(locator: &Locator) -> Result<usize> {
    match locator.layers.last() {
        Some(LocatorLayer::DatMember { index }) => Ok(*index),
        _ => Err(format!(
            "子节点不是 DAT 成员: {}",
            describe_locator(locator)
        )),
    }
}

fn script_dat_member(locator: &Locator) -> Result<usize> {
    child_dat_member(locator)
}

fn child_fdi_path(locator: &Locator) -> Result<String> {
    match locator.layers.last() {
        Some(LocatorLayer::FdiFile { path }) => Ok(path.clone()),
        _ => Err(format!(
            "子节点不是 FDI 文件: {}",
            describe_locator(locator)
        )),
    }
}

fn script_fdi_path(locator: &Locator) -> Result<String> {
    child_fdi_path(locator)
}

fn container_filename(locator: &Locator) -> String {
    match locator.layers.last() {
        Some(LocatorLayer::FdiFile { path }) => leaf_name(path).unwrap_or("disk.fdi").to_string(),
        Some(LocatorLayer::DatMember { index }) => format!("member_{index:03}.bin"),
        None => leaf_name(&locator.root).unwrap_or("source.bin").to_string(),
    }
}

fn describe_locator(locator: &Locator) -> String {
    let mut output = locator.root.clone();
    for layer in &locator.layers {
        match layer {
            LocatorLayer::FdiFile { path } => output.push_str(&format!(" -> FDI:{path}")),
            LocatorLayer::DatMember { index } => output.push_str(&format!(" -> DAT:#{index}")),
        }
    }
    output
}

fn leaf_name(path: &str) -> Result<&str> {
    path.rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("路径没有文件名: {path:?}"))
}

fn prepared_child_count(
    prepared: &BTreeMap<String, Vec<u8>>,
    id: usize,
    kind: ContainerKind,
) -> usize {
    let prefix = match kind {
        ContainerKind::FdiFat12 => format!("extract/fdi_{id:04}/"),
        ContainerKind::DriDat => format!("extract/dat_{id:04}/members/"),
    };
    prepared
        .keys()
        .filter(|path| path.starts_with(&prefix))
        .count()
}

fn insert_prepared(
    prepared: &mut BTreeMap<String, Vec<u8>>,
    path: String,
    bytes: Vec<u8>,
) -> Result<()> {
    validate_relative_path(&path)?;
    if prepared.insert(path.clone(), bytes).is_some() {
        return Err(format!("工作区输出路径重复: {path}"));
    }
    Ok(())
}

fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|error| format!("无法序列化 JSON: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes =
        fs::read(path).map_err(|error| format!("无法读取 JSON {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("无法解析 UTF-8 JSON {}: {error}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:X}", Sha256::digest(bytes))
}

fn verify_sha(bytes: &[u8], expected: &str, label: &str) -> Result<()> {
    let actual = sha256_hex(bytes);
    if actual != expected {
        return Err(format!(
            "{label} 的源 SHA-256 不匹配: expected={expected}, actual={actual}"
        ));
    }
    Ok(())
}

fn manifest_join(prefix: &str, relative: &str) -> Result<String> {
    validate_relative_path(prefix)?;
    validate_relative_path(relative)?;
    Ok(format!("{prefix}/{relative}"))
}

fn workspace_join(root: &Path, relative: &str) -> Result<PathBuf> {
    manifest_path_join(root, relative)
}

fn manifest_path_join(root: &Path, relative: &str) -> Result<PathBuf> {
    validate_relative_path(relative)?;
    let mut output = root.to_path_buf();
    for segment in relative.split('/') {
        output.push(segment);
    }
    Ok(output)
}

fn validate_relative_path(path: &str) -> Result<()> {
    if path.is_empty() || path.starts_with('/') || path.starts_with('\\') {
        return Err(format!("非法相对路径: {path:?}"));
    }
    for segment in path.split('/') {
        if segment.is_empty()
            || segment == "."
            || segment == ".."
            || segment.contains(['\\', ':'])
            || segment.chars().any(|character| character < ' ')
        {
            return Err(format!("非法相对路径: {path:?}"));
        }
    }
    Ok(())
}

fn validate_source_kind(source: &Path, expected: SourceKind) -> Result<()> {
    let actual = if source.is_file() {
        SourceKind::File
    } else if source.is_dir() {
        SourceKind::Directory
    } else {
        return Err(format!("源路径不存在: {}", source.display()));
    };
    if actual != expected {
        return Err("当前源路径类型与工作区提取时不一致".to_string());
    }
    Ok(())
}

fn validate_input_output(input: &Path, output: &Path) -> Result<()> {
    if !input.exists() {
        return Err(format!("输入不存在: {}", input.display()));
    }
    let input_absolute = fs::canonicalize(input)
        .map_err(|error| format!("无法解析输入路径 {}: {error}", input.display()))?;
    let output_absolute = absolute_lexical(output)?;
    if output_absolute == input_absolute
        || (input.is_dir() && output_absolute.starts_with(&input_absolute))
    {
        return Err("输出不能等于或位于输入目录内部".to_string());
    }
    Ok(())
}

fn absolute_lexical(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path)
            .map_err(|error| format!("无法解析路径 {}: {error}", path.display()));
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("路径没有父目录: {}", path.display()))?;
    let parent = if parent.exists() {
        fs::canonicalize(parent)
            .map_err(|error| format!("无法解析父目录 {}: {error}", parent.display()))?
    } else if parent.is_absolute() {
        parent.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("无法读取当前目录: {error}"))?
            .join(parent)
    };
    let name = path
        .file_name()
        .ok_or_else(|| format!("路径缺少名称: {}", path.display()))?;
    Ok(parent.join(name))
}

fn validate_workspace_output(output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return Ok(());
    }
    if !overwrite {
        return Err(format!(
            "输出已存在，需明确 --overwrite: {}",
            output.display()
        ));
    }
    if !output.is_dir() {
        return Err(format!("工作区输出不是目录: {}", output.display()));
    }
    let manifest: WorkspaceManifest = read_json(&output.join("workspace.json"))?;
    if manifest._format != WORKSPACE_FORMAT {
        return Err("已有目录不是本工具递归工作区，拒绝覆盖".to_string());
    }
    Ok(())
}

fn write_workspace(
    output: &Path,
    overwrite: bool,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    let staging = staging_path(output, "extract")?;
    fs::create_dir_all(&staging)
        .map_err(|error| format!("无法创建临时工作区 {}: {error}", staging.display()))?;
    let result = (|| {
        for (relative, bytes) in files {
            let path = manifest_path_join(&staging, relative)?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("无法创建 {}: {error}", parent.display()))?;
            }
            fs::write(&path, bytes)
                .map_err(|error| format!("无法写入 {}: {error}", path.display()))?;
        }
        commit_staging(&staging, output, overwrite)
    })();
    if result.is_err() && staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn write_packed_source(
    source: &Path,
    output: &Path,
    kind: SourceKind,
    replacements: &BTreeMap<String, Vec<u8>>,
    overwrite: bool,
    font_tmp: Option<&[u8]>,
) -> Result<Option<PathBuf>> {
    match kind {
        SourceKind::File => {
            if replacements.len() != 1 {
                return Err("单文件源必须恰好生成一个根输出".to_string());
            }
            let font_output = font_tmp.map(|_| sibling_font_path(output)).transpose()?;
            if let Some(path) = &font_output {
                if path.exists() && !overwrite {
                    return Err(format!(
                        "字库输出已存在，需明确 --overwrite: {}",
                        path.display()
                    ));
                }
            }
            let bytes = replacements.values().next().expect("one replacement");
            write_atomic_file(output, bytes, overwrite)?;
            if let (Some(font), Some(path)) = (font_tmp, &font_output) {
                write_atomic_file(path, font, overwrite)?;
            }
            Ok(font_output)
        }
        SourceKind::Directory => {
            if output.exists() && !overwrite {
                return Err(format!(
                    "输出已存在，需明确 --overwrite: {}",
                    output.display()
                ));
            }
            let staging = staging_path(output, "pack")?;
            fs::create_dir_all(&staging)
                .map_err(|error| format!("无法创建临时输出 {}: {error}", staging.display()))?;
            let result = (|| {
                copy_source_tree(source, source, &staging, replacements)?;
                if let Some(font) = font_tmp {
                    fs::write(staging.join("font.tmp"), font).map_err(|error| {
                        format!(
                            "无法写入生成字库 {}: {error}",
                            staging.join("font.tmp").display()
                        )
                    })?;
                }
                commit_staging(&staging, output, overwrite)
            })();
            if result.is_err() && staging.exists() {
                let _ = fs::remove_dir_all(&staging);
            }
            result?;
            Ok(font_tmp.map(|_| output.join("font.tmp")))
        }
    }
}

fn sibling_font_path(output: &Path) -> Result<PathBuf> {
    let parent = output
        .parent()
        .ok_or_else(|| format!("输出没有父目录: {}", output.display()))?;
    let stem = output
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("输出名无法表示为 Unicode: {}", output.display()))?;
    Ok(parent.join(format!("{stem}_font.tmp")))
}

fn copy_source_tree(
    root: &Path,
    current: &Path,
    output_root: &Path,
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    let mut entries = fs::read_dir(current)
        .map_err(|error| format!("无法读取源目录 {}: {error}", current.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("无法枚举源目录 {}: {error}", current.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry
            .file_type()
            .map_err(|error| format!("无法读取文件类型 {}: {error}", entry.path().display()))?;
        if file_type.is_symlink() {
            return Err(format!("源目录不接受符号链接: {}", entry.path().display()));
        }
        let relative_path = entry
            .path()
            .strip_prefix(root)
            .map_err(|_| "计算输出相对路径失败".to_string())?
            .to_path_buf();
        let destination = output_root.join(&relative_path);
        if file_type.is_dir() {
            fs::create_dir_all(&destination)
                .map_err(|error| format!("无法创建目录 {}: {error}", destination.display()))?;
            copy_source_tree(root, &entry.path(), output_root, replacements)?;
        } else if file_type.is_file() {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("无法创建目录 {}: {error}", parent.display()))?;
            }
            let key = relative_path
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            if let Some(bytes) = replacements.get(&key) {
                fs::write(&destination, bytes)
                    .map_err(|error| format!("无法写入 {}: {error}", destination.display()))?;
            } else {
                fs::copy(entry.path(), &destination)
                    .map_err(|error| format!("无法复制到 {}: {error}", destination.display()))?;
            }
        }
    }
    Ok(())
}

fn write_atomic_file(output: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    if output.exists() && !overwrite {
        return Err(format!(
            "输出已存在，需明确 --overwrite: {}",
            output.display()
        ));
    }
    let parent = output
        .parent()
        .ok_or_else(|| format!("输出没有父目录: {}", output.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("无法创建输出父目录 {}: {error}", parent.display()))?;
    let staging = staging_path(output, "file")?;
    fs::write(&staging, bytes)
        .map_err(|error| format!("无法写入临时文件 {}: {error}", staging.display()))?;
    if output.exists() {
        let backup = backup_path(output)?;
        fs::rename(output, &backup)
            .map_err(|error| format!("无法备份已有输出 {}: {error}", output.display()))?;
        if let Err(error) = fs::rename(&staging, output) {
            let _ = fs::rename(&backup, output);
            return Err(format!("提交输出失败，已尝试恢复旧文件: {error}"));
        }
        fs::remove_file(&backup)
            .map_err(|error| format!("输出已提交但无法删除备份 {}: {error}", backup.display()))?;
    } else {
        fs::rename(&staging, output)
            .map_err(|error| format!("无法提交输出 {}: {error}", output.display()))?;
    }
    Ok(())
}

fn staging_path(output: &Path, role: &str) -> Result<PathBuf> {
    let parent = output
        .parent()
        .ok_or_else(|| format!("输出没有父目录: {}", output.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("无法创建输出父目录 {}: {error}", parent.display()))?;
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("输出名无法表示为 Unicode: {}", output.display()))?;
    let path = parent.join(format!(".{name}.{role}-{}", std::process::id()));
    if path.exists() {
        return Err(format!("事务路径已存在: {}", path.display()));
    }
    Ok(path)
}

fn backup_path(output: &Path) -> Result<PathBuf> {
    let parent = output
        .parent()
        .ok_or_else(|| format!("输出没有父目录: {}", output.display()))?;
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("输出名无法表示为 Unicode: {}", output.display()))?;
    let path = parent.join(format!(".{name}.backup-{}", std::process::id()));
    if path.exists() {
        return Err(format!("备份路径已存在: {}", path.display()));
    }
    Ok(path)
}

fn commit_staging(staging: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if !output.exists() {
        return fs::rename(staging, output)
            .map_err(|error| format!("无法提交输出 {}: {error}", output.display()));
    }
    if !overwrite {
        return Err(format!("输出已存在: {}", output.display()));
    }
    let backup = backup_path(output)?;
    fs::rename(output, &backup)
        .map_err(|error| format!("无法备份旧输出 {}: {error}", output.display()))?;
    if let Err(error) = fs::rename(staging, output) {
        let _ = fs::rename(&backup, output);
        return Err(format!("提交新输出失败，已尝试恢复旧输出: {error}"));
    }
    if backup.is_dir() {
        fs::remove_dir_all(&backup)
            .map_err(|error| format!("新输出已提交但无法删除备份 {}: {error}", backup.display()))?;
    } else {
        fs::remove_file(&backup)
            .map_err(|error| format!("新输出已提交但无法删除备份 {}: {error}", backup.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locator_parent_is_stable() {
        let locator = Locator {
            root: "disk.fdi".to_string(),
            layers: vec![
                LocatorLayer::FdiFile {
                    path: "ADISK.DAT".to_string(),
                },
                LocatorLayer::DatMember { index: 3 },
            ],
        };
        let parent = parent_locator(&locator).unwrap();
        assert_eq!(parent.layers.len(), 1);
        assert_eq!(child_dat_member(&locator).unwrap(), 3);
    }

    #[test]
    fn rejects_parent_segments_in_manifest_paths() {
        assert!(validate_relative_path("safe/path").is_ok());
        assert!(validate_relative_path("../escape").is_err());
        assert!(validate_relative_path("safe/../escape").is_err());
    }

    #[test]
    fn dat_extract_directory_includes_safe_source_name() {
        assert_eq!(dat_extract_dir(9, "ACG.DAT"), "extract/dat_0009_ACG_DAT");
        assert_eq!(
            dat_extract_dir(12, "画像 資源.DAT"),
            "extract/dat_0012_画像_資源_DAT"
        );
    }

    #[test]
    fn old_workspace_container_without_source_file_remains_readable() {
        let record: ContainerRecord = serde_json::from_str(
            r#"{
                "id": 9,
                "kind": "dri_dat",
                "locator": {"root": "disk.fdi", "layers": []},
                "source_sha256": "00",
                "extract_dir": "extract/dat_0009",
                "children": 55
            }"#,
        )
        .unwrap();
        assert!(record.source_file.is_empty());
        assert_eq!(record.extract_dir, "extract/dat_0009");
    }
}
