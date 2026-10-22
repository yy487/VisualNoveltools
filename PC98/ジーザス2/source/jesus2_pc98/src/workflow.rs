//! End-to-end export, translation binding, font planning, and FDI rebuild.

use crate::{
    binary::{self, MenuString, SpeakerTable},
    disk::{self, FileIdentity, RebuiltDisk, SourceDisk},
    json_bytes,
    mes::{self, MesDocument, TextUnit, TextUnitKind, UnitLocator},
    output::ProtectedInput,
    Prepared, Result,
};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};
use vn_font::font_98::{self, EncodingPlan, EncodingPlanEntry, SubstitutionMap};
use vn_text::Entry;

const WORKSPACE_FORMAT: &str = "jesus2-pc98-workspace-v4";
const PRIMARY_FORMAT: &str = "jesus2-pc98-primary-v3";
const IMPLICIT_WAIT_SUFFIX: &str = "[[WAIT]]\n";
const ENGINE_PROGRAMS: [&str; 5] = [
    "JESUS2/J2K.COM",
    "JESUS2/J2M.COM",
    "JESUS2/J2O.COM",
    "JESUS2/J2Z.COM",
    "J2_GRP/J2E.COM",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WorkspaceManifest {
    format: String,
    tool_version: String,
    disks: Vec<WorkspaceDisk>,
    translations: Vec<TranslationBinding>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WorkspaceDisk {
    index: usize,
    source_size: usize,
    files: Vec<FileIdentity>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TranslationBinding {
    path: String,
    source: TranslationSource,
    entries: Vec<TranslationEntryBinding>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum TranslationSource {
    Mes {
        disk_index: usize,
        member_path: String,
    },
    BootMenu {
        disk_index: usize,
        member_path: String,
    },
    Speakers {
        executables: Vec<BoundMember>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct BoundMember {
    disk_index: usize,
    member_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TranslationEntryBinding {
    locator: Option<UnitLocator>,
    id: Option<String>,
    part_index: Option<usize>,
    speaker_index: Option<usize>,
    separator_before: String,
    restore_suffix: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimaryExtraction {
    format: String,
    disks: Vec<PrimaryDisk>,
    mes_resources: Vec<PrimaryMesResource>,
    boot_menu: Vec<MenuString>,
    speakers: Vec<String>,
    speaker_tables: Vec<PrimarySpeakerTable>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimaryDisk {
    index: usize,
    file_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimaryMesResource {
    disk_index: usize,
    member_path: String,
    units: Vec<TextUnit>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimarySpeakerTable {
    disk_index: usize,
    member_path: String,
    pointer_offset: usize,
    strings_offset: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildReport {
    format: String,
    source_disks: Vec<BuildDiskReport>,
    selected_translation_files: Vec<String>,
    changed_translation_entries: usize,
    rebuilt_members: usize,
    font_face: String,
    font_glyphs: usize,
    mapping: Vec<EncodingPlanEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildDiskReport {
    output_name: String,
    changed_files: usize,
    allocated_clusters: usize,
    released_clusters: usize,
}

#[derive(Clone, Debug)]
struct MesResource {
    disk_index: usize,
    member_path: String,
    document: MesDocument,
    units: Vec<TextUnit>,
}

#[derive(Clone, Debug)]
struct SpeakerResource {
    disk_index: usize,
    member_path: String,
    table: SpeakerTable,
}

#[derive(Clone, Debug)]
struct GameSnapshot {
    disks: Vec<SourceDisk>,
    mes: Vec<MesResource>,
    boot_menu_disk: usize,
    boot_menu_path: String,
    boot_menu_bytes: Vec<u8>,
    boot_menu: Vec<MenuString>,
    speakers: Vec<String>,
    speaker_resources: Vec<SpeakerResource>,
}

#[derive(Clone, Debug)]
struct LoadedTranslation {
    binding: TranslationBinding,
    entries: Vec<Entry>,
}

#[derive(Clone, Debug)]
struct DialogueSegment {
    part_index: usize,
    speaker_index: Option<usize>,
    separator_before: String,
    message: String,
    restore_suffix: Option<String>,
}

struct BuildProducts<'a> {
    changed_entries: usize,
    rebuilt: &'a [RebuiltDisk],
    face: &'a str,
    font_glyphs: usize,
    mapping: Vec<EncodingPlanEntry>,
}

type DiskReplacements = Vec<BTreeMap<String, Vec<u8>>>;
type ApplyResult = (DiskReplacements, usize);

pub fn prepare_export(input_directory: &Path, output: &Path) -> Result<Prepared> {
    let disks = disk::load_disks_from_directory(input_directory)?;
    let snapshot = parse_game(disks)?;
    let primary = primary_extraction(&snapshot);
    let primary_bytes = json_bytes(&primary)?;
    let manifest = make_manifest(&snapshot);
    let mut files = BTreeMap::new();

    for source_disk in &snapshot.disks {
        let prefix = format!("source/disks/disk_{:02}", source_disk.index + 1);
        for (member_path, bytes) in &source_disk.files {
            files.insert(format!("{prefix}/{member_path}"), bytes.clone());
        }
    }
    files.insert("primary/extraction.json".into(), primary_bytes);
    for binding in &manifest.translations {
        let entries = project_entries(&snapshot, binding)?;
        files.insert(
            binding.path.clone(),
            vn_text::write_template(&entries).map_err(|e| e.to_string())?,
        );
    }
    files.insert("jesus2_pc98_workspace.json".into(), json_bytes(&manifest)?);

    Prepared::new(
        source_stamps(&snapshot.disks),
        output,
        files,
        crate::OutputMetrics {
            disks: snapshot.disks.len(),
            files: snapshot.disks.iter().map(|disk| disk.files.len()).sum(),
            text_units: manifest
                .translations
                .iter()
                .map(|binding| binding.entries.len())
                .sum(),
            translation_files: manifest.translations.len(),
            changed_entries: 0,
            font_glyphs: 0,
        },
    )
}

pub fn prepare_import(
    input_directory: &Path,
    workspace_path: &Path,
    translation_paths: &[PathBuf],
    font_path: Option<&Path>,
    face: &str,
    output: &Path,
) -> Result<Prepared> {
    let workspace = resolve_workspace(workspace_path)?;
    let manifest_path = workspace.join("jesus2_pc98_workspace.json");
    let manifest_bytes = read_regular_file(&manifest_path, "工作区清单")?;
    let manifest: WorkspaceManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("工作区清单 JSON 无效: {error}"))?;
    if manifest.format != WORKSPACE_FORMAT {
        return Err(format!("不支持的工作区格式: {}", manifest.format));
    }

    let disks = disk::load_disks_from_directory(input_directory)?;
    let snapshot = parse_game(disks)?;
    let current_manifest = make_manifest(&snapshot);
    if manifest.format != current_manifest.format
        || manifest.disks != current_manifest.disks
        || manifest.translations != current_manifest.translations
    {
        return Err("工作区与当前五张原盘的盘序、文件清单或文本结构不匹配".into());
    }

    let translations = load_translations(&workspace, translation_paths, &snapshot, &manifest)?;
    let final_speakers = merge_speaker_labels(&snapshot, &translations)?;
    let final_texts = build_final_texts(&snapshot, &translations, &final_speakers)?;
    let reserved_cp932 = collect_reserved_cp932(&final_texts);
    let substitutions =
        SubstitutionMap::embedded().map_err(|error| format!("读取 PC-98 字库映射失败: {error}"))?;
    let plan = EncodingPlan::build_with_forbidden_cp932(
        &substitutions,
        reserved_cp932.iter().copied(),
        [],
        final_texts.iter().map(String::as_str),
    )
    .map_err(|error| format!("无法建立文本/字库编码计划: {error}"))?;

    let mut external_sources = vec![crate::output::directory_input(&workspace)?];
    let font_source = match font_path {
        Some(path) => {
            let resolved = disk::resolve_source(path, "NP2 font.tmp")?;
            let bytes =
                fs::read(&resolved).map_err(|error| format!("读取 font.tmp 失败: {error}"))?;
            external_sources.push(ProtectedInput { path: resolved });
            bytes
        }
        None => font_98::EMBEDDED_FONT.to_vec(),
    };
    font_98::validate_font(&font_source).map_err(|error| format!("font.tmp 格式无效: {error}"))?;
    let font_build = font_98::prepare_font(&font_source, &plan.requests(), &reserved_cp932, face)
        .map_err(|error| format!("重绘 PC-98 字库失败: {error}"))?;
    let mapping = plan
        .manifest_entries()
        .map_err(|error| format!("生成字库映射清单失败: {error}"))?;

    let (replacements, changed_entries) =
        apply_translations(&snapshot, &translations, &final_speakers, &plan)?;
    let rebuilt = disk::rebuild_disks(&snapshot.disks, &replacements)?;
    let report = build_report(
        &snapshot,
        &translations,
        BuildProducts {
            changed_entries,
            rebuilt: &rebuilt,
            face,
            font_glyphs: font_build.patched_glyphs,
            mapping,
        },
    );

    let mut files = BTreeMap::new();
    for disk in &rebuilt {
        files.insert(
            format!("disks/disk_{:02}.FDI", disk.index + 1),
            disk.bytes.clone(),
        );
    }
    files.insert("font.tmp".into(), font_build.bytes);
    files.insert("font_mapping.json".into(), json_bytes(&report.mapping)?);
    files.insert("build_report.json".into(), json_bytes(&report)?);

    let font_glyphs = report.font_glyphs;
    let mut sources = source_stamps(&snapshot.disks);
    sources.extend(external_sources);
    Prepared::new(
        sources,
        output,
        files,
        crate::OutputMetrics {
            disks: snapshot.disks.len(),
            files: snapshot.disks.iter().map(|disk| disk.files.len()).sum(),
            text_units: manifest
                .translations
                .iter()
                .map(|binding| binding.entries.len())
                .sum(),
            translation_files: translations.len(),
            changed_entries,
            font_glyphs,
        },
    )
}

fn parse_game(disks: Vec<SourceDisk>) -> Result<GameSnapshot> {
    let mut mes = Vec::new();
    for source_disk in &disks {
        for (member_path, bytes) in source_disk.files_with_extension("MES") {
            let document = mes::parse(bytes).map_err(|error| {
                format!(
                    "第 {} 张盘的 {member_path} 解析失败: {error}",
                    source_disk.index + 1
                )
            })?;
            let units = document.units();
            mes.push(MesResource {
                disk_index: source_disk.index,
                member_path: member_path.to_owned(),
                document,
                units,
            });
        }
    }
    mes.sort_by(|left, right| {
        (left.disk_index, left.member_path.to_ascii_uppercase())
            .cmp(&(right.disk_index, right.member_path.to_ascii_uppercase()))
    });
    if mes.len() != 16 {
        return Err(format!(
            "预期解析 16 个已确认的 JESUS2 MES 资源，实际为 {} 个",
            mes.len()
        ));
    }
    let mut mes_file_names = BTreeSet::new();
    for resource in &mes {
        let file_name = Path::new(&resource.member_path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("MES 文件名不是有效 UTF-8: {}", resource.member_path))?;
        if !mes_file_names.insert(file_name.to_ascii_uppercase()) {
            return Err(format!("MES 基名重复，不能扁平放置翻译 JSON: {file_name}"));
        }
    }
    let actual_programs = disks
        .iter()
        .flat_map(|source_disk| {
            source_disk
                .files_with_extension("PRG")
                .map(|(path, _)| format!("{}:{}", source_disk.index, path.to_ascii_uppercase()))
        })
        .collect::<BTreeSet<_>>();
    let mut expected_programs = BTreeSet::new();
    for resource in &mes {
        let program_path = Path::new(&resource.member_path)
            .with_extension("PRG")
            .to_str()
            .ok_or_else(|| format!("MES 路径无法映射到 PRG: {}", resource.member_path))?
            .to_owned();
        disks[resource.disk_index]
            .file(&program_path)
            .map_err(|_| {
                format!(
                    "第 {} 张盘缺少与 {} 配对的 PRG",
                    resource.disk_index + 1,
                    resource.member_path
                )
            })?;
        expected_programs.insert(format!(
            "{}:{}",
            resource.disk_index,
            program_path.to_ascii_uppercase()
        ));
    }
    if actual_programs != expected_programs {
        return Err(format!(
            "PRG 程序集合与 16 个 MES 配对不一致：找到 {} 个 PRG，预期 {} 个",
            actual_programs.len(),
            expected_programs.len()
        ));
    }

    let (boot_menu_disk, boot_menu_path, boot_menu_bytes) = find_member(&disks, "JESUS2.COM")?;
    let boot_menu = binary::find_boot_menu(&boot_menu_bytes)
        .map_err(|error| format!("启动菜单定位失败: {error}"))?;

    let mut speaker_resources = Vec::with_capacity(ENGINE_PROGRAMS.len());
    for program in ENGINE_PROGRAMS {
        let (disk_index, member_path, bytes) = find_member(&disks, program)?;
        let table = binary::find_speaker_table(&bytes)
            .map_err(|error| format!("{member_path} 的说话人表定位失败: {error}"))?;
        speaker_resources.push(SpeakerResource {
            disk_index,
            member_path,
            table,
        });
    }
    let speakers = normalized_speaker_messages(&speaker_resources[0].table);
    if speakers.len() != binary::SPEAKER_COUNT {
        return Err("说话人表条目数不是 24".into());
    }
    for resource in speaker_resources.iter().skip(1) {
        if normalized_speaker_messages(&resource.table) != speakers {
            return Err(format!(
                "{} 的说话人表与 J2K.COM 不一致，拒绝跨程序同步修改",
                resource.member_path
            ));
        }
    }

    Ok(GameSnapshot {
        disks,
        mes,
        boot_menu_disk,
        boot_menu_path,
        boot_menu_bytes,
        boot_menu,
        speakers,
        speaker_resources,
    })
}

fn normalized_speaker_messages(table: &SpeakerTable) -> Vec<String> {
    binary::speaker_messages(table)
        .iter()
        .map(|name| mes::normalize_halfwidth_katakana(name))
        .collect()
}

fn find_member(disks: &[SourceDisk], wanted: &str) -> Result<(usize, String, Vec<u8>)> {
    let mut matches = Vec::new();
    for source_disk in disks {
        for (path, bytes) in &source_disk.files {
            if path.eq_ignore_ascii_case(wanted) {
                matches.push((source_disk.index, path.clone(), bytes.clone()));
            }
        }
    }
    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err(format!("五张原盘中缺少成员 {wanted}")),
        count => Err(format!("五张原盘中成员 {wanted} 有 {count} 份，定位不唯一")),
    }
}

fn primary_extraction(snapshot: &GameSnapshot) -> PrimaryExtraction {
    PrimaryExtraction {
        format: PRIMARY_FORMAT.into(),
        disks: snapshot
            .disks
            .iter()
            .map(|disk| PrimaryDisk {
                index: disk.index,
                file_count: disk.files.len(),
            })
            .collect(),
        mes_resources: snapshot
            .mes
            .iter()
            .map(|resource| PrimaryMesResource {
                disk_index: resource.disk_index,
                member_path: resource.member_path.clone(),
                units: resource.units.clone(),
            })
            .collect(),
        boot_menu: snapshot.boot_menu.clone(),
        speakers: snapshot.speakers.clone(),
        speaker_tables: snapshot
            .speaker_resources
            .iter()
            .map(|resource| PrimarySpeakerTable {
                disk_index: resource.disk_index,
                member_path: resource.member_path.clone(),
                pointer_offset: resource.table.pointer_offset,
                strings_offset: resource.table.strings_offset,
            })
            .collect(),
    }
}

fn make_manifest(snapshot: &GameSnapshot) -> WorkspaceManifest {
    let mut translations = Vec::new();
    for resource in &snapshot.mes {
        let mut entries = Vec::new();
        for unit in &resource.units {
            let segments = split_dialogue(unit);
            for segment in segments {
                entries.push(TranslationEntryBinding {
                    locator: Some(unit.locator.clone()),
                    id: None,
                    part_index: Some(segment.part_index),
                    speaker_index: segment.speaker_index,
                    separator_before: segment.separator_before,
                    restore_suffix: segment.restore_suffix,
                });
            }
        }
        translations.push(TranslationBinding {
            path: mes_translation_path(resource.disk_index, &resource.member_path),
            source: TranslationSource::Mes {
                disk_index: resource.disk_index,
                member_path: resource.member_path.clone(),
            },
            entries,
        });
    }
    translations.push(TranslationBinding {
        path: "translations/system_menu.json".into(),
        source: TranslationSource::BootMenu {
            disk_index: snapshot.boot_menu_disk,
            member_path: snapshot.boot_menu_path.clone(),
        },
        entries: snapshot
            .boot_menu
            .iter()
            .map(|menu| TranslationEntryBinding {
                locator: None,
                id: Some(menu.id.clone()),
                part_index: None,
                speaker_index: None,
                separator_before: String::new(),
                restore_suffix: None,
            })
            .collect(),
    });
    translations.push(TranslationBinding {
        path: "translations/speakers.json".into(),
        source: TranslationSource::Speakers {
            executables: snapshot
                .speaker_resources
                .iter()
                .map(|resource| BoundMember {
                    disk_index: resource.disk_index,
                    member_path: resource.member_path.clone(),
                })
                .collect(),
        },
        entries: (0..binary::SPEAKER_COUNT)
            .map(|index| TranslationEntryBinding {
                locator: None,
                id: Some(format!("speaker_{index:02}")),
                part_index: None,
                speaker_index: Some(index),
                separator_before: String::new(),
                restore_suffix: None,
            })
            .collect(),
    });
    WorkspaceManifest {
        format: WORKSPACE_FORMAT.into(),
        tool_version: env!("CARGO_PKG_VERSION").into(),
        disks: snapshot
            .disks
            .iter()
            .map(|disk| WorkspaceDisk {
                index: disk.index,
                source_size: disk.image.len(),
                files: disk.identities.clone(),
            })
            .collect(),
        translations,
    }
}

fn mes_translation_path(_disk_index: usize, member_path: &str) -> String {
    let file_name = member_path.rsplit('/').next().unwrap_or(member_path);
    format!("translations/{file_name}.json")
}

fn project_entries(snapshot: &GameSnapshot, binding: &TranslationBinding) -> Result<Vec<Entry>> {
    match &binding.source {
        TranslationSource::Mes {
            disk_index,
            member_path,
            ..
        } => {
            let resource = find_mes(snapshot, *disk_index, member_path)?;
            let mut result = Vec::with_capacity(binding.entries.len());
            let mut cursor = 0usize;
            for unit in &resource.units {
                for segment in split_dialogue(unit) {
                    let entry_binding = binding
                        .entries
                        .get(cursor)
                        .ok_or_else(|| format!("{member_path} 的工作区分段定位数量少于源对白"))?;
                    result.push(projected_dialogue_entry(
                        snapshot,
                        unit,
                        &segment,
                        entry_binding,
                    )?);
                    cursor += 1;
                }
            }
            if cursor != binding.entries.len() {
                return Err(format!("{member_path} 的工作区分段定位数量多于源对白"));
            }
            Ok(result)
        }
        TranslationSource::BootMenu { .. } => Ok(snapshot
            .boot_menu
            .iter()
            .map(|field| Entry {
                name: None,
                message: field.message.clone(),
            })
            .collect()),
        TranslationSource::Speakers { .. } => Ok(snapshot
            .speakers
            .iter()
            .map(|speaker| Entry {
                name: None,
                message: speaker.clone(),
            })
            .collect()),
    }
}

fn speaker_marker_at(input: &str) -> Option<(usize, usize)> {
    let marker = input.strip_prefix("[[SPK:")?;
    let digits = marker.get(..2)?;
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) || !marker[2..].starts_with("]]") {
        return None;
    }
    let index = digits.parse::<usize>().ok()?;
    (index < binary::SPEAKER_COUNT).then_some((index, 10))
}

fn split_dialogue(unit: &TextUnit) -> Vec<DialogueSegment> {
    let mut segments = Vec::new();
    let mut current = DialogueSegment {
        part_index: 0,
        speaker_index: None,
        separator_before: String::new(),
        message: String::new(),
        restore_suffix: None,
    };
    let mut cursor = 0usize;
    while cursor < unit.message.len() {
        let rest = &unit.message[cursor..];
        if unit.kind == TextUnitKind::Message && rest.starts_with(IMPLICIT_WAIT_SUFFIX) {
            current.restore_suffix = Some(IMPLICIT_WAIT_SUFFIX.to_owned());
            segments.push(current);
            current = DialogueSegment {
                part_index: segments.len(),
                speaker_index: None,
                separator_before: String::new(),
                message: String::new(),
                restore_suffix: None,
            };
            cursor += IMPLICIT_WAIT_SUFFIX.len();
            continue;
        }
        if let Some((speaker_index, marker_length)) = speaker_marker_at(rest) {
            if current.message.is_empty()
                && current.speaker_index.is_none()
                && current.separator_before.is_empty()
            {
                current.speaker_index = Some(speaker_index);
                cursor += marker_length;
                continue;
            }
            let separator_before = if unit.message[..cursor].ends_with('\n') {
                current.message.pop();
                "\n".to_owned()
            } else {
                String::new()
            };
            segments.push(current);
            current = DialogueSegment {
                part_index: segments.len(),
                speaker_index: Some(speaker_index),
                separator_before,
                message: String::new(),
                restore_suffix: None,
            };
            cursor += marker_length;
            continue;
        }
        let Some(character) = rest.chars().next() else {
            break;
        };
        current.message.push(character);
        cursor += character.len_utf8();
    }
    if !segments.is_empty()
        && current.message.is_empty()
        && current.speaker_index.is_none()
        && current.separator_before.is_empty()
        && current.restore_suffix.is_none()
    {
        return segments;
    }
    segments.push(current);
    segments
}

fn projected_dialogue_entry(
    snapshot: &GameSnapshot,
    unit: &TextUnit,
    segment: &DialogueSegment,
    binding: &TranslationEntryBinding,
) -> Result<Entry> {
    if binding.locator.as_ref() != Some(&unit.locator)
        || binding.part_index != Some(segment.part_index)
        || binding.speaker_index != segment.speaker_index
        || binding.separator_before != segment.separator_before
    {
        return Err(format!(
            "MES record key 0x{:04X}: 分段定位元数据不匹配",
            unit.record_key
        ));
    }
    if binding.restore_suffix != segment.restore_suffix {
        return Err(format!(
            "MES record key 0x{:04X}: 隐藏等待符元数据不匹配",
            unit.record_key
        ));
    }
    let name = binding
        .speaker_index
        .map(|index| {
            snapshot
                .speakers
                .get(index)
                .cloned()
                .ok_or_else(|| format!("说话人索引 {index} 超出 24 项表"))
        })
        .transpose()?;
    Ok(Entry {
        name,
        message: segment.message.clone(),
    })
}

fn materialize_dialogue_unit(
    snapshot: &GameSnapshot,
    unit: &TextUnit,
    segments: &[DialogueSegment],
    entries: &[Entry],
    bindings: &[TranslationEntryBinding],
) -> Result<String> {
    if segments.len() != entries.len() || segments.len() != bindings.len() {
        return Err(format!(
            "MES record key 0x{:04X}: 对话分段数量不匹配",
            unit.record_key
        ));
    }
    let mut result = String::new();
    for ((segment, entry), binding) in segments.iter().zip(entries).zip(bindings) {
        projected_dialogue_entry(snapshot, unit, segment, binding)?;
        let mut message = mes::normalize_halfwidth_katakana(&entry.message);
        if let Some(suffix) = &binding.restore_suffix {
            if message.ends_with("[[WAIT]]") || message.ends_with(IMPLICIT_WAIT_SUFFIX) {
                return Err(
                    "末尾等待符由工具自动补回，请移除该 name 行 message 末尾的 [[WAIT]]".into(),
                );
            }
            message.push_str(suffix);
        }
        result.push_str(&segment.separator_before);
        if let Some(index) = binding.speaker_index {
            result.push_str(&format!("[[SPK:{index:02}]]"));
        }
        result.push_str(&message);
    }
    Ok(result)
}

fn find_mes<'a>(
    snapshot: &'a GameSnapshot,
    disk_index: usize,
    member_path: &str,
) -> Result<&'a MesResource> {
    snapshot
        .mes
        .iter()
        .find(|resource| {
            resource.disk_index == disk_index
                && resource.member_path.eq_ignore_ascii_case(member_path)
        })
        .ok_or_else(|| format!("一次提取中找不到 MES 资源 {member_path}"))
}

fn source_stamps(disks: &[SourceDisk]) -> Vec<ProtectedInput> {
    disks
        .iter()
        .map(|disk| ProtectedInput {
            path: disk.source_path.clone(),
        })
        .collect()
}

fn resolve_workspace(path: &Path) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("读取工作区失败 {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("工作区必须是非符号链接目录".into());
    }
    fs::canonicalize(path).map_err(|error| format!("解析工作区路径失败: {error}"))
}

fn read_regular_file(path: &Path, label: &str) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("读取{label}失败 {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("{label}必须是普通文件: {}", path.display()));
    }
    fs::read(path).map_err(|error| format!("读取{label}失败 {}: {error}", path.display()))
}

fn load_translations(
    workspace: &Path,
    paths: &[PathBuf],
    snapshot: &GameSnapshot,
    manifest: &WorkspaceManifest,
) -> Result<Vec<LoadedTranslation>> {
    if paths.is_empty() {
        return Err("至少选择一个翻译 JSON 文件或 translations 目录".into());
    }
    let mut selected = BTreeSet::new();
    let mut selected_root = false;
    let translation_root = fs::canonicalize(workspace.join("translations"))
        .map_err(|error| format!("工作区 translations 目录缺失: {error}"))?;
    for supplied in paths {
        let metadata = fs::symlink_metadata(supplied)
            .map_err(|error| format!("翻译输入不可读取 {}: {error}", supplied.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("翻译输入不接受符号链接: {}", supplied.display()));
        }
        let canonical = fs::canonicalize(supplied)
            .map_err(|error| format!("解析翻译路径失败 {}: {error}", supplied.display()))?;
        if metadata.is_dir() {
            if canonical != translation_root {
                return Err(format!(
                    "目录输入必须是本工作区的 translations 目录: {}",
                    supplied.display()
                ));
            }
            selected_root = true;
            continue;
        }
        if !metadata.is_file() {
            return Err(format!("翻译输入不是普通文件: {}", supplied.display()));
        }
        let relative = manifest
            .translations
            .iter()
            .find_map(|binding| {
                let expected = workspace.join(&binding.path);
                let matches = fs::canonicalize(&expected).is_ok_and(|path| path == canonical);
                matches.then_some(binding.path.clone())
            })
            .ok_or_else(|| format!("翻译文件不属于此工作区: {}", supplied.display()))?;
        selected.insert(relative);
    }
    if selected_root {
        selected.extend(
            manifest
                .translations
                .iter()
                .map(|binding| binding.path.clone()),
        );
    }
    if selected.is_empty() {
        return Err("未选中工作区中的翻译 JSON".into());
    }

    let by_path = manifest
        .translations
        .iter()
        .map(|binding| (binding.path.as_str(), binding))
        .collect::<HashMap<_, _>>();
    let mut translations = Vec::with_capacity(selected.len());
    for relative in selected {
        let binding = by_path
            .get(relative.as_str())
            .ok_or_else(|| format!("清单没有登记翻译文件 {relative}"))?;
        let expected = project_entries(snapshot, binding)?;
        let bytes = read_regular_file(&workspace.join(&binding.path), "翻译 JSON")?;
        let entries = vn_text::read_template(&bytes)
            .map_err(|error| format!("{relative} 翻译 JSON 无效: {error}"))?;
        vn_text::validate_shape(&expected, &entries)
            .map_err(|error| format!("{relative} 条目形状不匹配: {error}"))?;
        translations.push(LoadedTranslation {
            binding: (*binding).clone(),
            entries,
        });
    }
    Ok(translations)
}

fn build_final_texts(
    snapshot: &GameSnapshot,
    translations: &[LoadedTranslation],
    speakers: &[String],
) -> Result<Vec<String>> {
    let by_path = translations
        .iter()
        .map(|item| (item.binding.path.as_str(), item))
        .collect::<HashMap<_, _>>();
    let mut texts = Vec::new();
    for resource in &snapshot.mes {
        let path = mes_translation_path(resource.disk_index, &resource.member_path);
        if let Some(translation) = by_path.get(path.as_str()) {
            let mut cursor = 0usize;
            for unit in &resource.units {
                let segments = split_dialogue(unit);
                let end = cursor + segments.len();
                let entries = translation
                    .entries
                    .get(cursor..end)
                    .ok_or_else(|| format!("{path} 的译文分段不足"))?;
                let bindings = translation
                    .binding
                    .entries
                    .get(cursor..end)
                    .ok_or_else(|| format!("{path} 的分段定位不足"))?;
                let message =
                    materialize_dialogue_unit(snapshot, unit, &segments, entries, bindings)?;
                if message != unit.message {
                    resource.document.encode_template(
                        &unit.locator,
                        &message,
                        placeholder_cp932,
                    )?;
                }
                texts.push(font_text(&message));
                cursor = end;
            }
            if cursor != translation.entries.len() {
                return Err(format!("{path} 的译文分段多于源对白"));
            }
        } else {
            for unit in &resource.units {
                texts.push(font_text(&unit.message));
            }
        }
    }
    let menu_binding = "translations/system_menu.json";
    if let Some(translation) = by_path.get(menu_binding) {
        texts.extend(
            translation
                .entries
                .iter()
                .map(|entry| normalize_display_text(&entry.message)),
        );
    } else {
        texts.extend(
            snapshot
                .boot_menu
                .iter()
                .map(|field| normalize_display_text(&field.message)),
        );
    }

    for label in speakers {
        let cells = speaker_cells(label)?;
        texts.push(normalize_display_text(&cells));
    }
    Ok(texts)
}

fn merge_speaker_labels(
    snapshot: &GameSnapshot,
    translations: &[LoadedTranslation],
) -> Result<Vec<String>> {
    let mut updates = BTreeMap::<usize, String>::new();
    for translation in translations {
        match &translation.binding.source {
            TranslationSource::Mes { .. } => {
                for (entry, binding) in translation.entries.iter().zip(&translation.binding.entries)
                {
                    let Some(index) = binding.speaker_index else {
                        continue;
                    };
                    let source_name = snapshot
                        .speakers
                        .get(index)
                        .ok_or_else(|| format!("说话人索引 {index} 超出 24 项表"))?;
                    let translated_name = entry
                        .name
                        .as_ref()
                        .ok_or_else(|| format!("说话人索引 {index} 缺少 name 字段"))?;
                    let translated_name = mes::normalize_halfwidth_katakana(translated_name);
                    record_speaker_update(&mut updates, index, source_name, &translated_name)?;
                }
            }
            TranslationSource::Speakers { .. } => {
                for (entry, binding) in translation.entries.iter().zip(&translation.binding.entries)
                {
                    let index = binding
                        .speaker_index
                        .ok_or_else(|| "speakers.json 缺少稳定说话人索引".to_owned())?;
                    let source_name = snapshot
                        .speakers
                        .get(index)
                        .ok_or_else(|| format!("说话人索引 {index} 超出 24 项表"))?;
                    let translated_name = mes::normalize_halfwidth_katakana(&entry.message);
                    record_speaker_update(&mut updates, index, source_name, &translated_name)?;
                }
            }
            TranslationSource::BootMenu { .. } => {}
        }
    }

    let mut result = snapshot.speakers.clone();
    for (index, label) in updates {
        speaker_cells(&label)?;
        result[index] = label;
    }
    Ok(result)
}

fn record_speaker_update(
    updates: &mut BTreeMap<usize, String>,
    index: usize,
    source_name: &str,
    translated_name: &str,
) -> Result<()> {
    if translated_name == source_name {
        return Ok(());
    }
    if let Some(previous) = updates.insert(index, translated_name.to_owned()) {
        if previous != translated_name {
            return Err(format!(
                "说话人 #{index:02} 收到冲突译名 {previous:?} 与 {translated_name:?}"
            ));
        }
    }
    Ok(())
}

fn placeholder_cp932(text: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(text.chars().count() * 2);
    for _ in text.chars() {
        bytes.extend([0x81, 0x40]);
    }
    Ok(bytes)
}

fn collect_reserved_cp932(texts: &[String]) -> BTreeSet<u16> {
    let mut reserved = BTreeSet::new();
    for text in texts {
        for character in text.chars() {
            let value = character.to_string();
            let (bytes, _, had_errors) = SHIFT_JIS.encode(&value);
            if !had_errors && bytes.len() == 2 && font_98::has_loaded_np2_slot(character) {
                reserved.insert(u16::from_be_bytes([bytes[0], bytes[1]]));
            }
        }
    }
    reserved
}

fn normalize_display_text(text: &str) -> String {
    mes::normalize_halfwidth_katakana(text)
        .chars()
        .map(|character| match character {
            ' ' => '　',
            '!'..='~' => char::from_u32(character as u32 + 0xFEE0).unwrap_or(character),
            _ => character,
        })
        .collect()
}

fn font_text(template: &str) -> String {
    let visible = mes::plain_text(template)
        .chars()
        .filter(|character| !matches!(*character, '\r' | '\n'))
        .collect::<String>();
    normalize_display_text(&visible)
}

fn encode_display_text(plan: &EncodingPlan, text: &str) -> Result<Vec<u8>> {
    plan.encode_cp932(&normalize_display_text(text))
        .map_err(|error| format!("CP932/字库编码失败: {error}"))
}

fn speaker_cells(label: &str) -> Result<String> {
    let label = mes::normalize_halfwidth_katakana(label);
    if label.chars().count() > 4 || label.chars().any(char::is_control) {
        return Err(format!("说话人名称最多 4 个全角字格: {label:?}"));
    }
    let mut cells = label;
    cells.extend(std::iter::repeat_n('　', 4 - cells.chars().count()));
    cells.push('「');
    Ok(cells)
}

fn apply_translations(
    snapshot: &GameSnapshot,
    translations: &[LoadedTranslation],
    final_speakers: &[String],
    plan: &EncodingPlan,
) -> Result<ApplyResult> {
    let mut replacements = vec![BTreeMap::new(); snapshot.disks.len()];
    let mut changed_entries = 0usize;
    for translation in translations {
        match &translation.binding.source {
            TranslationSource::Mes {
                disk_index,
                member_path,
                ..
            } => {
                let resource = find_mes(snapshot, *disk_index, member_path)?;
                let mut body_replacements = BTreeMap::<UnitLocator, Vec<u8>>::new();
                let mut cursor = 0usize;
                for unit in &resource.units {
                    let segments = split_dialogue(unit);
                    let end = cursor + segments.len();
                    let entries = translation
                        .entries
                        .get(cursor..end)
                        .ok_or_else(|| format!("{member_path} 的译文分段不足"))?;
                    let bindings = translation
                        .binding
                        .entries
                        .get(cursor..end)
                        .ok_or_else(|| format!("{member_path} 的分段定位不足"))?;
                    for ((segment, entry), binding) in segments.iter().zip(entries).zip(bindings) {
                        let expected = projected_dialogue_entry(snapshot, unit, segment, binding)?;
                        let translated_name = entry
                            .name
                            .as_ref()
                            .map(|name| mes::normalize_halfwidth_katakana(name));
                        if entry.message != expected.message
                            || translated_name.as_ref() != expected.name.as_ref()
                        {
                            changed_entries += 1;
                        }
                    }
                    let message =
                        materialize_dialogue_unit(snapshot, unit, &segments, entries, bindings)?;
                    if message != unit.message {
                        let encoded =
                            resource
                                .document
                                .encode_template(&unit.locator, &message, |text| {
                                    encode_display_text(plan, text)
                                })?;
                        body_replacements.insert(unit.locator.clone(), encoded);
                    }
                    cursor = end;
                }
                if cursor != translation.entries.len() {
                    return Err(format!("{member_path} 的译文分段多于源对白"));
                }
                let rebuilt = resource.document.rebuild(&body_replacements)?;
                let source = snapshot.disks[*disk_index].file(member_path)?;
                if rebuilt != source {
                    replacements[*disk_index].insert(member_path.clone(), rebuilt);
                }
            }
            TranslationSource::BootMenu {
                disk_index,
                member_path,
                ..
            } => {
                let mut by_id = BTreeMap::new();
                for (field, entry) in snapshot.boot_menu.iter().zip(&translation.entries) {
                    if entry.message != field.message {
                        changed_entries += 1;
                        by_id.insert(field.id.clone(), entry.message.clone());
                    }
                }
                let patched = binary::patch_boot_menu(
                    &snapshot.boot_menu_bytes,
                    &snapshot.boot_menu,
                    &by_id,
                    |text| encode_display_text(plan, text),
                )?;
                let original = snapshot.disks[*disk_index].file(member_path)?;
                if patched != original {
                    replacements[*disk_index].insert(member_path.clone(), patched);
                }
            }
            TranslationSource::Speakers { executables } => {
                if executables.len() != snapshot.speaker_resources.len() {
                    return Err("工作区说话人可执行文件绑定数量异常".into());
                }
                for (binding, resource) in executables.iter().zip(&snapshot.speaker_resources) {
                    if binding.disk_index != resource.disk_index
                        || !binding
                            .member_path
                            .eq_ignore_ascii_case(&resource.member_path)
                    {
                        return Err(format!("说话人可执行文件绑定错位: {}", binding.member_path));
                    }
                }
                for (old, new) in snapshot.speakers.iter().zip(&translation.entries) {
                    if old != &new.message {
                        changed_entries += 1;
                    }
                }
            }
        }
    }

    if final_speakers != snapshot.speakers {
        for resource in &snapshot.speaker_resources {
            let original = snapshot.disks[resource.disk_index].file(&resource.member_path)?;
            let patched =
                binary::patch_speaker_table(original, &resource.table, final_speakers, |text| {
                    encode_display_text(plan, text)
                })?;
            if patched != original {
                replacements[resource.disk_index].insert(resource.member_path.clone(), patched);
            }
        }
    }
    Ok((replacements, changed_entries))
}

fn build_report(
    snapshot: &GameSnapshot,
    translations: &[LoadedTranslation],
    products: BuildProducts<'_>,
) -> BuildReport {
    BuildReport {
        format: "jesus2-pc98-build-v1".into(),
        source_disks: products
            .rebuilt
            .iter()
            .map(|disk| BuildDiskReport {
                output_name: format!("disk_{:02}.FDI", disk.index + 1),
                changed_files: disk.report.changed_files,
                allocated_clusters: disk.report.allocated_clusters,
                released_clusters: disk.report.released_clusters,
            })
            .collect(),
        selected_translation_files: translations
            .iter()
            .map(|translation| translation.binding.path.clone())
            .collect(),
        changed_translation_entries: products.changed_entries,
        rebuilt_members: snapshot.disks.iter().map(|disk| disk.files.len()).sum(),
        font_face: products.face.to_owned(),
        font_glyphs: products.font_glyphs,
        mapping: products.mapping,
    }
}
