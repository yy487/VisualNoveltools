//! JIS text extraction/injection for DOLL, BASE and MIKI.
//!
//! The adapter owns record parsing and RIO53 control-code handling. PC-88
//! KANJI1 carrier planning and ROM patching are delegated to `vn-font::font_88`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Parameters, PreparedOperation, Preview, Progress,
    RunReport, Value,
};
use vn_text::Entry;

use crate::font::{self, Pc88FontBuild};
use crate::n88::ResourceFile;
use crate::{n88, Result};

const ESC_K: [u8; 2] = [0x1B, 0x4B];
const JIS_R: u16 = 0x2372;
const JIS_E: u16 = 0x2365;
const JIS_N: u16 = 0x236E;
const JIS_C: u16 = 0x2363;
const JIS_I: u16 = 0x2369;
const JIS_PLUS: u16 = 0x215C;
const JIS_MINUS: u16 = 0x215D;
const JIS_EQUALS: u16 = 0x2161;
const BASE_SLOT_BYTES: usize = 85;
const DISK_RECORD_BYTES: usize = 256;
const DOLL_FIXED_REFERENCE_RECORDS: usize = 288;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextOperation {
    Row,
    End,
    NextPage,
    Graphic {
        id: u8,
    },
    Variable {
        index: u8,
        op: VariableOperation,
        value: u8,
    },
    RawPair {
        value: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableOperation {
    Add,
    Subtract,
    Assign,
}

impl TextOperation {
    fn escape(&self) -> String {
        match self {
            Self::Row => "\\n".into(),
            Self::End => "\\e".into(),
            Self::NextPage => "\\p".into(),
            Self::Graphic { id } => format!("\\c{id:02}"),
            Self::Variable { index, op, value } => {
                let operator = match op {
                    VariableOperation::Add => "+",
                    VariableOperation::Subtract => "-",
                    VariableOperation::Assign => "=",
                };
                format!("\\i{index:02}{operator}{value}")
            }
            Self::RawPair { value } => format!("\\x{value:04X}"),
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(self, Self::End | Self::NextPage)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextPrimaryManifest {
    pub schema: String,
    pub source_container_sha256: String,
    pub disks: Vec<TextDiskIdentity>,
    pub resources: Vec<TextResourceManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextDiskIdentity {
    pub label: String,
    pub file_name: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextResourceManifest {
    pub disk_label: String,
    pub file_name: String,
    pub sha256: String,
    pub length: usize,
    pub layout: String,
    pub slots: Vec<TextSlotManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextSlotManifest {
    pub index: usize,
    pub offset: usize,
    pub capacity: usize,
    pub editable: bool,
    pub source_sha256: String,
    pub message: Option<String>,
    pub control_sequence: Vec<TextOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextBuildReport {
    pub source_container_sha256: String,
    pub rebuilt_container_sha256: String,
    pub changed_records: usize,
    pub total_records: usize,
    pub doll_pages_inserted: usize,
    pub resource_lengths: Vec<(String, usize)>,
    pub font: crate::font::Pc88FontReport,
}

#[derive(Debug, Clone)]
struct TextSlot {
    primary: TextSlotManifest,
    original: Vec<u8>,
    preserved_suffix: Vec<TextOperation>,
    post_next_page_e: bool,
    final_nul: bool,
}

#[derive(Debug, Clone)]
struct TextResource {
    disk_index: usize,
    file: ResourceFile,
    primary: TextResourceManifest,
    slots: Vec<TextSlot>,
}

#[derive(Debug, Clone)]
struct TextCatalog {
    primary: TextPrimaryManifest,
    entries: Vec<Entry>,
    entry_slots: Vec<(usize, usize)>,
    resources: Vec<TextResource>,
    original_double_byte_codes: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MessagePart {
    Text(String),
    Control(TextOperation),
}

#[derive(Debug, Clone)]
struct ParsedTranslation {
    parts: Vec<MessagePart>,
    control_sequence: Vec<TextOperation>,
    canonical_message: String,
    display_texts: Vec<String>,
}

#[derive(Debug, Clone)]
struct TextBuild {
    outer_d88: Vec<u8>,
    disk_bytes: Vec<Vec<u8>>,
    kanji1_rom: Vec<u8>,
    font: Pc88FontBuild,
    report: TextBuildReport,
}

#[derive(Debug, Default)]
pub struct ExportTextOperation;

impl Operation for ExportTextOperation {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "export-text",
            "提取 DOLL / BASE / MIKI 文本",
            vec![
                Field::new("workspace", "已拆分的 A/B/C D88 工作目录", FieldKind::Path).required(),
                Field::new("output", "文本模板输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "按本作 RIO53 reader 解码 ESC K / JIS 文本，控制码以反斜杠转义导出。".into();
        spec.primary = true;
        spec.writes = true;
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(path) = paths.first() {
            if parameters.get("workspace").is_none() {
                parameters.set("workspace", Value::Path(path.clone()));
            }
            if parameters.get("output").is_none() {
                parameters.set("output", Value::Path(path.join("text_export")));
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let workspace = parameters.path("workspace")?.to_path_buf();
        let output = parameters.path("output")?.to_path_buf();
        let overwrite = parameters.flag("overwrite");
        crate::validate_output(&output, overwrite).map_err(vn_cli::Error::from)?;
        crate::ensure_paths_disjoint(&workspace, &output).map_err(vn_cli::Error::from)?;

        let prepared =
            crate::prepare_resource_workspace(&workspace).map_err(vn_cli::Error::from)?;
        let catalog = TextCatalog::from_prepared(&prepared).map_err(vn_cli::Error::from)?;
        let primary_bytes = json_bytes(&catalog.primary)?;
        let translation_bytes = vn_text::write_template(&catalog.entries)
            .map_err(|error| vn_cli::Error::from(format!("生成 translation.json 失败：{error}")))?;
        let files = BTreeMap::from([
            (PathBuf::from("primary.json"), primary_bytes),
            (PathBuf::from("translation.json"), translation_bytes),
        ]);
        let details = catalog
            .primary
            .resources
            .iter()
            .map(|resource| {
                format!(
                    "disk_{}/{}：{} 个槽，{} 个可译项，{} 字节",
                    resource.disk_label,
                    resource.file_name,
                    resource.slots.len(),
                    resource.slots.iter().filter(|slot| slot.editable).count(),
                    resource.length
                )
            })
            .collect();
        let preview = Preview {
            inputs: vec![workspace.clone()],
            outputs: vec![output.clone()],
            steps: vec![
                "重验 A/B/C D88 与来源清单".into(),
                "读取 MIKI 固定块、BASE 三字段菜单和 DOLL 256 字节文本块".into(),
                "解析 JIS 控制码并保留非文本块、填充和原始身份".into(),
                "写出来源绑定的 primary.json 与 translation.json".into(),
            ],
            details,
        };
        Ok(Box::new(PreparedTextExport {
            prepared,
            output,
            overwrite,
            files,
            preview,
            record_count: catalog.entries.len(),
        }))
    }
}

struct PreparedTextExport {
    prepared: crate::PreparedResources,
    output: PathBuf,
    overwrite: bool,
    files: BTreeMap<PathBuf, Vec<u8>>,
    preview: Preview,
    record_count: usize,
}

impl PreparedOperation for PreparedTextExport {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        progress.report("复核来源 D88 工作目录")?;
        crate::verify_resource_inputs(&self.prepared).map_err(vn_cli::Error::from)?;
        progress.report("事务式写出文本模板")?;
        write_output_directory(&self.output, &self.files, self.overwrite)
            .map_err(vn_cli::Error::from)?;
        Ok(RunReport {
            summary: format!("文本导出完成：{} 条记录。", self.record_count),
            totals: vec![
                ("文本记录".into(), self.record_count as u64),
                ("输出文件".into(), self.files.len() as u64),
            ],
            outputs: vec![self.output],
            warnings: Vec::new(),
        })
    }
}

#[derive(Debug, Default)]
pub struct ImportTextOperation;

impl Operation for ImportTextOperation {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "import-text",
            "注回文本并重建 D88 / PC-88 字库",
            vec![
                Field::new("workspace", "原始 A/B/C D88 工作目录", FieldKind::Path).required(),
                Field::new("primary", "对应来源的 primary.json", FieldKind::Path).required(),
                Field::new("translation", "编辑后的 translation.json", FieldKind::Path).required(),
                Field::new("output", "重建输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "验证来源身份与控制码后，用 PC-88 动态映射重建文本和内置 KANJI1.ROM；DOLL 文本可自动分页、重映射 DES2 并扩展簇链。".into();
        spec.primary = true;
        spec.writes = true;
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(path) = paths.first() {
            if parameters.get("workspace").is_none() {
                parameters.set("workspace", Value::Path(path.clone()));
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let workspace = parameters.path("workspace")?.to_path_buf();
        let primary_path = parameters.path("primary")?.to_path_buf();
        let translation_path = parameters.path("translation")?.to_path_buf();
        let output = parameters.path("output")?.to_path_buf();
        let overwrite = parameters.flag("overwrite");
        crate::validate_output(&output, overwrite).map_err(vn_cli::Error::from)?;
        crate::ensure_paths_disjoint(&workspace, &output).map_err(vn_cli::Error::from)?;

        let prepared =
            crate::prepare_resource_workspace(&workspace).map_err(vn_cli::Error::from)?;
        let catalog = TextCatalog::from_prepared(&prepared).map_err(vn_cli::Error::from)?;
        let primary_bytes =
            fs::read(&primary_path).map_err(|error| format!("读取 primary.json 失败：{error}"))?;
        let translation_bytes = fs::read(&translation_path)
            .map_err(|error| format!("读取 translation.json 失败：{error}"))?;
        let supplied_primary: TextPrimaryManifest = serde_json::from_slice(&primary_bytes)
            .map_err(|error| format!("primary.json 格式无效：{error}"))?;
        if supplied_primary != catalog.primary {
            return Err("primary.json 与当前 D88 的哈希、记录边界或顺序不匹配；请重新导出".into());
        }
        let translations = vn_text::read_template(&translation_bytes)
            .map_err(|error| format!("translation.json 格式无效：{error}"))?;
        validate_translations(&catalog.entries, &translations, &catalog)
            .map_err(vn_cli::Error::from)?;
        let build = build_translated_disks(&prepared, &catalog, &translations)
            .map_err(vn_cli::Error::from)?;
        let files = build_output_files(&build, &primary_bytes, &translation_bytes)?;
        let preview = Preview {
            inputs: vec![
                workspace.clone(),
                primary_path.clone(),
                translation_path.clone(),
            ],
            outputs: vec![output.clone()],
            steps: vec![
                "重新解析原始盘并完整比较 primary.json".into(),
                "校验固定字段控制码；DOLL 允许调整换行并按需增加续页".into(),
                "以原文占用和最终译文共同生成一份 PC-88 动态字库计划".into(),
                "注回 JIS 文本、更新 DES2 页号、扩展 DOLL 簇链并重建 KANJI1.ROM".into(),
                "重新解析三盘与文本记录，核对注回内容和源映射".into(),
            ],
            details: vec![
                format!(
                    "文本记录：{}；本次改写：{}；DOLL 新增续页：{}；动态载体：{}。",
                    build.report.total_records,
                    build.report.changed_records,
                    build.report.doll_pages_inserted,
                    build.report.font.plan.mapping_used.len()
                ),
                format!(
                    "内置 KANJI1.ROM：{} 字节，来源 SHA-256 {}。",
                    build.kanji1_rom.len(),
                    build.report.font.source_rom_sha256
                ),
                format!(
                    "重建 D88 SHA-256：{}。",
                    build.report.rebuilt_container_sha256
                ),
            ],
        };
        Ok(Box::new(PreparedTextImport {
            prepared,
            primary_path,
            primary_snapshot: primary_bytes,
            translation_path,
            translation_snapshot: translation_bytes,
            output,
            overwrite,
            files,
            preview,
            report: build.report,
        }))
    }
}

struct PreparedTextImport {
    prepared: crate::PreparedResources,
    primary_path: PathBuf,
    primary_snapshot: Vec<u8>,
    translation_path: PathBuf,
    translation_snapshot: Vec<u8>,
    output: PathBuf,
    overwrite: bool,
    files: BTreeMap<PathBuf, Vec<u8>>,
    preview: Preview,
    report: TextBuildReport,
}

impl PreparedOperation for PreparedTextImport {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        progress.report("复核 D88、primary.json 与 translation.json 快照")?;
        crate::verify_resource_inputs(&self.prepared).map_err(vn_cli::Error::from)?;
        verify_file_snapshot(&self.primary_path, &self.primary_snapshot, "primary.json")
            .map_err(vn_cli::Error::from)?;
        verify_file_snapshot(
            &self.translation_path,
            &self.translation_snapshot,
            "translation.json",
        )
        .map_err(vn_cli::Error::from)?;
        progress.report("事务式写出重建 D88、字库和清单")?;
        write_output_directory(&self.output, &self.files, self.overwrite)
            .map_err(vn_cli::Error::from)?;
        Ok(RunReport {
            summary: "文本注回与 PC-88 动态字库重建完成".into(),
            totals: vec![
                ("文本记录".into(), self.report.total_records as u64),
                ("改写记录".into(), self.report.changed_records as u64),
                (
                    "DOLL 新增续页".into(),
                    self.report.doll_pages_inserted as u64,
                ),
                (
                    "动态字槽".into(),
                    self.report.font.plan.mapping_used.len() as u64,
                ),
                ("输出文件".into(), self.files.len() as u64),
            ],
            outputs: vec![self.output],
            warnings: Vec::new(),
        })
    }
}

fn verify_file_snapshot(path: &Path, expected: &[u8], label: &str) -> Result<()> {
    let actual = fs::read(path).map_err(|error| format!("重新读取 {label} 失败：{error}"))?;
    if actual != expected {
        return Err(format!("{label} 在预检后发生变化；未写出重建结果"));
    }
    Ok(())
}

fn write_output_directory(
    output: &Path,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    overwrite: bool,
) -> Result<()> {
    let stage = crate::create_unique_dir(crate::output_parent(output), output, "stage")?;
    let write_result = (|| {
        for (relative, bytes) in files {
            if relative.is_absolute()
                || relative.components().any(|part| {
                    matches!(
                        part,
                        std::path::Component::ParentDir | std::path::Component::RootDir
                    )
                })
            {
                return Err(format!("不安全的输出相对路径：{}", relative.display()));
            }
            crate::write_new_file(&stage.join(relative), bytes)?;
        }
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    if let Err(error) = crate::commit_stage(&stage, output, overwrite) {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    Ok(())
}

fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|error| format!("JSON 序列化失败：{error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

impl TextCatalog {
    fn from_prepared(prepared: &crate::PreparedResources) -> Result<Self> {
        let disks = prepared
            .disks
            .iter()
            .map(|disk| TextDiskIdentity {
                label: disk.label.to_owned(),
                file_name: disk.file_name.clone(),
                sha256: disk.parsed.summary.source_sha256.clone(),
            })
            .collect::<Vec<_>>();
        let mut resources = Vec::new();
        let mut entries = Vec::new();
        let mut entry_slots = Vec::new();
        let mut original_double_byte_codes = BTreeSet::new();

        for spec in [
            TextResourceSpec {
                disk_label: "a",
                file_name: "MIKI",
                layout: TextResourceLayout::Fixed256 { final_nul: true },
            },
            TextResourceSpec {
                disk_label: "b",
                file_name: "BASE",
                layout: TextResourceLayout::BaseThreeFields,
            },
            TextResourceSpec {
                disk_label: "b",
                file_name: "DOLL",
                layout: TextResourceLayout::DollPages,
            },
        ] {
            let disk_index = prepared
                .disks
                .iter()
                .position(|disk| disk.label == spec.disk_label)
                .ok_or_else(|| format!("缺少 disk_{}", spec.disk_label))?;
            let disk = &prepared.disks[disk_index];
            let file = disk
                .parsed
                .files
                .iter()
                .find(|file| file.display_name.eq_ignore_ascii_case(spec.file_name))
                .cloned()
                .ok_or_else(|| {
                    format!("disk_{} 中缺少 {} 文件", spec.disk_label, spec.file_name)
                })?;
            let ranges = slot_ranges(spec.layout, file.data.len(), spec.file_name)?;
            let mut slots = Vec::with_capacity(ranges.len());
            let mut primary_slots = Vec::with_capacity(ranges.len());

            for (slot_index, (offset, capacity)) in ranges.into_iter().enumerate() {
                let end = offset
                    .checked_add(capacity)
                    .ok_or_else(|| format!("{} slot {slot_index} 偏移溢出", spec.file_name))?;
                let original = file
                    .data
                    .get(offset..end)
                    .ok_or_else(|| format!("{} slot {slot_index} 越过文件末尾", spec.file_name))?
                    .to_vec();
                let slot = parse_text_slot(
                    spec.disk_label,
                    spec.file_name,
                    slot_index,
                    offset,
                    original,
                    matches!(
                        spec.layout,
                        TextResourceLayout::Fixed256 { final_nul: true }
                            | TextResourceLayout::DollPages
                    ),
                    spec.file_name == "DOLL",
                )?;

                if slot.primary.editable {
                    let message = slot
                        .primary
                        .message
                        .as_deref()
                        .ok_or_else(|| "可编辑文本槽没有 message".to_owned())?;
                    let parsed_message = parse_translation_message(message).map_err(|error| {
                        format!(
                            "disk_{}/{} slot {slot_index} 原文控制码无效：{error}",
                            spec.disk_label, spec.file_name
                        )
                    })?;
                    for display_text in &parsed_message.display_texts {
                        for character in display_text.chars() {
                            if let Some(cp932) = font::strict_cp932_pair(character) {
                                original_double_byte_codes.insert(cp932);
                            }
                        }
                    }
                    entries.push(Entry {
                        name: Some(String::new()),
                        message: message.to_owned(),
                    });
                    entry_slots.push((resources.len(), slot_index));
                }
                primary_slots.push(slot.primary.clone());
                slots.push(slot);
            }

            let primary = TextResourceManifest {
                disk_label: spec.disk_label.to_owned(),
                file_name: spec.file_name.to_owned(),
                sha256: file.sha256.clone(),
                length: file.data.len(),
                layout: match spec.layout {
                    TextResourceLayout::Fixed256 { .. } => "fixed_256".into(),
                    TextResourceLayout::BaseThreeFields => "base_3x85_in_256".into(),
                    TextResourceLayout::DollPages => "doll_256_byte_pages".into(),
                },
                slots: primary_slots,
            };
            resources.push(TextResource {
                disk_index,
                file,
                primary,
                slots,
            });
        }

        let primary = TextPrimaryManifest {
            schema: "executive-road-pc88-text-primary-v1".into(),
            source_container_sha256: prepared.source_manifest.source_sha256.clone(),
            disks,
            resources: resources
                .iter()
                .map(|resource| resource.primary.clone())
                .collect(),
        };
        Ok(Self {
            primary,
            entries,
            entry_slots,
            resources,
            original_double_byte_codes: original_double_byte_codes.into_iter().collect(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum TextResourceLayout {
    Fixed256 { final_nul: bool },
    DollPages,
    BaseThreeFields,
}

#[derive(Debug, Clone, Copy)]
struct TextResourceSpec {
    disk_label: &'static str,
    file_name: &'static str,
    layout: TextResourceLayout,
}

fn slot_ranges(
    layout: TextResourceLayout,
    file_len: usize,
    file_name: &str,
) -> Result<Vec<(usize, usize)>> {
    match layout {
        TextResourceLayout::Fixed256 { .. } | TextResourceLayout::DollPages => {
            if file_len == 0 || file_len % DISK_RECORD_BYTES != 0 {
                return Err(format!("{file_name} 长度 {} 不是 0x100 的整数倍", file_len));
            }
            Ok((0..file_len)
                .step_by(DISK_RECORD_BYTES)
                .map(|offset| (offset, DISK_RECORD_BYTES))
                .collect())
        }
        TextResourceLayout::BaseThreeFields => {
            if file_len == 0 || file_len % DISK_RECORD_BYTES != 0 {
                return Err(format!("BASE 长度 {file_len} 不是 0x100 的整数倍"));
            }
            let blocks = file_len / DISK_RECORD_BYTES;
            let mut ranges = Vec::with_capacity(blocks * 3);
            for block in 0..blocks {
                let base = block * DISK_RECORD_BYTES;
                for field in 0..3 {
                    ranges.push((base + field * BASE_SLOT_BYTES, BASE_SLOT_BYTES));
                }
            }
            Ok(ranges)
        }
    }
}

fn parse_text_slot(
    disk_label: &str,
    file_name: &str,
    index: usize,
    offset: usize,
    original: Vec<u8>,
    final_nul: bool,
    allow_opaque: bool,
) -> Result<TextSlot> {
    parse_text_slot_with_suffix(
        disk_label,
        file_name,
        index,
        offset,
        original,
        final_nul,
        allow_opaque,
        None,
    )
}

fn parse_text_slot_with_suffix(
    disk_label: &str,
    file_name: &str,
    index: usize,
    offset: usize,
    original: Vec<u8>,
    final_nul: bool,
    allow_opaque: bool,
    preserved_suffix_hint: Option<&[TextOperation]>,
) -> Result<TextSlot> {
    let source_sha256 = crate::sha256_hex(&original);
    if !original.starts_with(&ESC_K) {
        if !allow_opaque {
            return Err(format!(
                "{} slot {index} 在 disk_{disk_label} 中没有 ESC K 标记",
                file_name
            ));
        }
        return Ok(TextSlot {
            primary: TextSlotManifest {
                index,
                offset,
                capacity: original.len(),
                editable: false,
                source_sha256,
                message: None,
                control_sequence: Vec::new(),
            },
            original,
            preserved_suffix: Vec::new(),
            post_next_page_e: false,
            final_nul,
        });
    }
    if final_nul && original.last() != Some(&0) {
        return Err(format!("{file_name} slot {index} 缺少固定记录末尾 NUL"));
    }

    let data_end = original.len() - usize::from(final_nul);
    let mut cursor = ESC_K.len();
    let mut message = String::new();
    let mut control_sequence = Vec::new();
    let mut terminal = None;

    while cursor + 1 < data_end {
        let pair = u16::from_be_bytes([original[cursor], original[cursor + 1]]);
        if let Some((operation, byte_len)) = parse_control(&original, cursor, data_end)? {
            let terminal_op = operation.is_terminal();
            message.push_str(&operation.escape());
            control_sequence.push(operation.clone());
            cursor += byte_len;
            if terminal_op {
                terminal = Some(operation);
                break;
            }
            continue;
        }
        let character = match font::jis_pair_to_char(pair) {
            Ok(character) => character,
            Err(_) => {
                let operation = TextOperation::RawPair { value: pair };
                message.push_str(&operation.escape());
                control_sequence.push(operation);
                cursor += 2;
                continue;
            }
        };
        message.push(character);
        cursor += 2;
    }

    let Some(terminal) = terminal else {
        return Err(format!("{file_name} slot {index} 没有 RIO53 终止操作 e/n"));
    };
    let post_next_page_e = matches!(&terminal, TextOperation::NextPage)
        && original[cursor..data_end]
            .windows(2)
            .any(|pair| pair == [0x23, 0x65]);
    let preserved_suffix = if let Some(hint) = preserved_suffix_hint {
        if !matches!(hint.last(), Some(operation) if operation.is_terminal())
            || !control_sequence.ends_with(hint)
        {
            return Err(format!("{file_name} slot {index} 固定结束码与记录不符"));
        }
        hint.to_vec()
    } else {
        let mut suffix = vec![terminal.clone()];
        let terminal_escape = terminal.escape();
        if !message.ends_with(&terminal_escape) {
            return Err(format!("{file_name} slot {index} 终止操作不在记录末尾"));
        }
        message.truncate(message.len() - terminal_escape.len());
        let row_escape = TextOperation::Row.escape();
        let mut control_index = control_sequence.len().saturating_sub(1);
        while control_index > 0
            && matches!(control_sequence[control_index - 1], TextOperation::Row)
            && message.ends_with(&row_escape)
        {
            message.truncate(message.len() - row_escape.len());
            suffix.insert(0, TextOperation::Row);
            control_index -= 1;
        }
        suffix
    };
    if preserved_suffix_hint.is_some() {
        for operation in preserved_suffix.iter().rev() {
            let escape = operation.escape();
            if !message.ends_with(&escape) {
                return Err(format!("{file_name} slot {index} 固定结束码不在预期位置"));
            }
            message.truncate(message.len() - escape.len());
        }
    }
    Ok(TextSlot {
        primary: TextSlotManifest {
            index,
            offset,
            capacity: original.len(),
            editable: true,
            source_sha256,
            message: Some(message),
            control_sequence,
        },
        original,
        preserved_suffix,
        post_next_page_e,
        final_nul,
    })
}

fn parse_control(
    bytes: &[u8],
    cursor: usize,
    end: usize,
) -> Result<Option<(TextOperation, usize)>> {
    if cursor + 1 >= end {
        return Ok(None);
    }
    let first = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]);
    let pair = |at: usize| -> Result<u16> {
        let raw = bytes
            .get(at..at + 2)
            .ok_or_else(|| "RIO53 控制码被截断".to_owned())?;
        Ok(u16::from_be_bytes([raw[0], raw[1]]))
    };
    let one = match first {
        JIS_R => Some((TextOperation::Row, 2)),
        JIS_E => Some((TextOperation::End, 2)),
        JIS_N => Some((TextOperation::NextPage, 2)),
        JIS_C => {
            let tens = jis_digit(pair(cursor + 2)?)?;
            let ones = jis_digit(pair(cursor + 4)?)?;
            Some((
                TextOperation::Graphic {
                    id: tens * 10 + ones,
                },
                6,
            ))
        }
        JIS_I => {
            let tens = jis_digit(pair(cursor + 2)?)?;
            let ones = jis_digit(pair(cursor + 4)?)?;
            let op = match pair(cursor + 6)? {
                JIS_PLUS => VariableOperation::Add,
                JIS_MINUS => VariableOperation::Subtract,
                JIS_EQUALS => VariableOperation::Assign,
                other => return Err(format!("RIO53 i 操作符 JIS {other:04X} 未知")),
            };
            let value = jis_digit(pair(cursor + 8)?)?;
            Some((
                TextOperation::Variable {
                    index: tens * 10 + ones,
                    op,
                    value,
                },
                10,
            ))
        }
        _ => None,
    };
    if let Some((_, length)) = &one {
        if cursor + *length > end {
            return Err("RIO53 控制码参数越过当前记录".into());
        }
    }
    Ok(one)
}

fn jis_digit(code: u16) -> Result<u8> {
    if (0x2330..=0x2339).contains(&code) {
        Ok((code - 0x2330) as u8)
    } else {
        Err(format!("RIO53 控制码预期全角数字，实际 JIS {code:04X}"))
    }
}

fn parse_translation_message(message: &str) -> Result<ParsedTranslation> {
    let bytes = message.as_bytes();
    let mut parts = Vec::new();
    let mut control_sequence = Vec::new();
    let mut canonical_message = String::new();
    let mut display_texts = Vec::new();
    let mut cursor = 0usize;
    let mut text_start = 0usize;

    while cursor < bytes.len() {
        if bytes[cursor] != b'\\' {
            cursor += 1;
            continue;
        }
        append_translation_text(
            &message[text_start..cursor],
            &mut parts,
            &mut canonical_message,
            &mut display_texts,
        )?;
        if cursor + 1 >= bytes.len() {
            return Err("译文末尾有未完成的反斜杠转义".into());
        }
        if bytes[cursor + 1] == b'\\' {
            append_translation_text("\\", &mut parts, &mut canonical_message, &mut display_texts)?;
            cursor += 2;
            text_start = cursor;
            continue;
        }
        let (operation, escape_len) = parse_control_escape(message, cursor)?;
        canonical_message.push_str(&operation.escape());
        control_sequence.push(operation.clone());
        parts.push(MessagePart::Control(operation));
        cursor += escape_len;
        text_start = cursor;
    }
    append_translation_text(
        &message[text_start..],
        &mut parts,
        &mut canonical_message,
        &mut display_texts,
    )?;

    if control_sequence.iter().any(TextOperation::is_terminal) {
        return Err("\\e 和 \\p 结束码由注入器从原记录保留，请勿写入译文".into());
    }
    Ok(ParsedTranslation {
        parts,
        control_sequence,
        canonical_message,
        display_texts,
    })
}

fn append_translation_text(
    source: &str,
    parts: &mut Vec<MessagePart>,
    canonical_message: &mut String,
    display_texts: &mut Vec<String>,
) -> Result<()> {
    if source.is_empty() {
        return Ok(());
    }
    let text = normalize_display_text(source)?;
    if !text.is_empty() {
        canonical_message.push_str(&text);
        display_texts.push(text.clone());
        parts.push(MessagePart::Text(text));
    }
    Ok(())
}

fn parse_control_escape(message: &str, cursor: usize) -> Result<(TextOperation, usize)> {
    let bytes = message.as_bytes();
    if bytes.get(cursor) != Some(&b'\\') {
        return Err("控制转义必须以反斜杠开头".into());
    }
    let tag = *bytes
        .get(cursor + 1)
        .ok_or_else(|| "译文末尾有未完成的反斜杠转义".to_owned())?;
    let operation = match tag {
        b'n' => return Ok((TextOperation::Row, 2)),
        b'e' => return Ok((TextOperation::End, 2)),
        b'p' => return Ok((TextOperation::NextPage, 2)),
        b'c' => {
            let id = escape_ascii_slice(message, cursor + 2, cursor + 4, "图像编号")?;
            TextOperation::Graphic {
                id: parse_two_ascii_digits(id, "图像编号")?,
            }
        }
        b'i' => {
            let index = escape_ascii_slice(message, cursor + 2, cursor + 4, "变量编号")?;
            let operator = *bytes
                .get(cursor + 4)
                .ok_or_else(|| "\\i 变量转义缺少操作符和值".to_owned())?;
            let value = *bytes
                .get(cursor + 5)
                .ok_or_else(|| "\\i 变量转义缺少操作符和值".to_owned())?;
            let op = match operator {
                b'+' => VariableOperation::Add,
                b'-' => VariableOperation::Subtract,
                b'=' => VariableOperation::Assign,
                _ => return Err("\\i 操作符必须是 +、- 或 =".into()),
            };
            let value = char::from(value).to_string();
            TextOperation::Variable {
                index: parse_two_ascii_digits(index, "变量编号")?,
                op,
                value: parse_one_ascii_digit(&value, "变量值")?,
            }
        }
        b'x' => {
            let value = escape_ascii_slice(message, cursor + 2, cursor + 6, "原始 JIS 字节")?;
            if !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err("\\x 原始 JIS 字节必须是四位十六进制".into());
            }
            TextOperation::RawPair {
                value: u16::from_str_radix(value, 16)
                    .map_err(|_| "\\x 原始 JIS 字节超出 16 位".to_owned())?,
            }
        }
        _ => return Err(format!("未知反斜杠转义 \\{}", char::from(tag))),
    };
    let length = match tag {
        b'c' => 4,
        b'i' | b'x' => 6,
        _ => unreachable!(),
    };
    Ok((operation, length))
}

fn escape_ascii_slice<'a>(
    message: &'a str,
    start: usize,
    end: usize,
    label: &str,
) -> Result<&'a str> {
    let value = message
        .get(start..end)
        .ok_or_else(|| format!("反斜杠转义缺少{label}"))?;
    if !value.is_ascii() {
        return Err(format!("{label}必须是 ASCII 字符"));
    }
    Ok(value)
}

fn parse_two_ascii_digits(text: &str, label: &str) -> Result<u8> {
    let bytes = text.as_bytes();
    if bytes.len() != 2 || !bytes.iter().all(u8::is_ascii_digit) {
        return Err(format!("{label}必须是两位 ASCII 数字"));
    }
    Ok((bytes[0] - b'0') * 10 + (bytes[1] - b'0'))
}

fn parse_one_ascii_digit(text: &str, label: &str) -> Result<u8> {
    let bytes = text.as_bytes();
    if bytes.len() != 1 || !bytes[0].is_ascii_digit() {
        return Err(format!("{label}必须是一位 ASCII 数字"));
    }
    Ok(bytes[0] - b'0')
}

/// The source renderer consumes JIS double-byte glyphs. Normalize pasted
/// half-width ASCII to the corresponding full-width JIS character before font
/// planning. RIO53 controls are recognized only through explicit backslash escapes.
fn normalize_display_text(text: &str) -> Result<String> {
    let mut output = String::with_capacity(text.len());
    for character in text.chars() {
        let normalized = match character {
            '\n' | '\r' => return Err("译文含实际换行；请使用 \\n 转义操作".into()),
            ' ' => '　',
            '!'..='~' => char::from_u32(0xFF01 + character as u32 - 0x21)
                .ok_or_else(|| "ASCII 字符无法转为全角 JIS".to_owned())?,
            other => other,
        };
        output.push(normalized);
    }
    Ok(output)
}

fn validate_translations(
    expected: &[Entry],
    supplied: &[Entry],
    catalog: &TextCatalog,
) -> Result<Vec<ParsedTranslation>> {
    vn_text::validate_shape(expected, supplied)
        .map_err(|error| format!("翻译 JSON 与来源不匹配：{error}"))?;
    if catalog.entry_slots.len() != expected.len() {
        return Err("primary 文本索引与翻译条目数不一致".into());
    }
    let mut parsed = Vec::with_capacity(supplied.len());
    for (index, (source, translation)) in expected.iter().zip(supplied).enumerate() {
        if source.name != translation.name {
            return Err(format!("第 {} 条翻译记录的稳定 name 被修改", index + 1));
        }
        let message = parse_translation_message(&translation.message)
            .map_err(|error| format!("第 {} 条译文：{error}", index + 1))?;
        let (resource_index, slot_index) = catalog.entry_slots[index];
        let slot = catalog
            .resources
            .get(resource_index)
            .and_then(|resource| resource.slots.get(slot_index))
            .ok_or_else(|| format!("第 {} 条翻译记录没有 source slot", index + 1))?;
        let is_doll = catalog.resources[resource_index].primary.file_name == "DOLL";
        let visible_controls = visible_control_sequence(slot);
        let source_controls = if is_doll {
            without_row_controls(visible_controls)
        } else {
            visible_controls.to_vec()
        };
        let translated_controls = if is_doll {
            without_row_controls(&message.control_sequence)
        } else {
            message.control_sequence.clone()
        };
        if translated_controls != source_controls {
            return Err(format!(
                "disk_{}/{} slot {} 的 RIO53 操作码被更改；DOLL 正文只允许增删 \\n 换行，固定结尾由工具保留",
                catalog.resources[resource_index].primary.disk_label,
                catalog.resources[resource_index].primary.file_name,
                slot_index
            ));
        }
        parsed.push(message);
    }
    Ok(parsed)
}

fn visible_control_sequence(slot: &TextSlot) -> &[TextOperation] {
    let visible_len = slot
        .primary
        .control_sequence
        .len()
        .saturating_sub(slot.preserved_suffix.len());
    &slot.primary.control_sequence[..visible_len]
}

fn without_row_controls(controls: &[TextOperation]) -> Vec<TextOperation> {
    controls
        .iter()
        .filter(|operation| !matches!(operation, TextOperation::Row))
        .cloned()
        .collect()
}

fn build_translated_disks(
    prepared: &crate::PreparedResources,
    catalog: &TextCatalog,
    translations: &[Entry],
) -> Result<TextBuild> {
    let parsed_translations = validate_translations(&catalog.entries, translations, catalog)?;
    let display_texts = parsed_translations
        .iter()
        .flat_map(|message| message.display_texts.iter().cloned())
        .collect::<Vec<_>>();
    let font_build = font::build_pc88_font(&catalog.original_double_byte_codes, &display_texts)?;

    let mut entry_by_slot = BTreeMap::<(usize, usize), usize>::new();
    for (entry_index, key) in catalog.entry_slots.iter().copied().enumerate() {
        entry_by_slot.insert(key, entry_index);
    }
    let mut replacement_resources = Vec::with_capacity(catalog.resources.len());
    let mut expected_files = vec![BTreeMap::<usize, Vec<u8>>::new(); prepared.disks.len()];
    let mut disk_replacements = vec![Vec::<n88::FileReplacement>::new(); prepared.disks.len()];
    let mut changed_records = 0usize;
    let mut doll_pages_inserted = 0usize;
    let mut doll_old_to_new = None::<Vec<usize>>;
    let mut doll_page_counts = None::<Vec<usize>>;
    for (resource_index, resource) in catalog.resources.iter().enumerate() {
        let is_doll = resource.primary.file_name == "DOLL";
        let replacement = if is_doll {
            if doll_old_to_new.is_some() {
                return Err("文本清单中出现多个 DOLL 资源".into());
            }
            let mut bytes = Vec::new();
            let mut old_to_new = vec![0usize; resource.slots.len()];
            let mut page_counts = vec![0usize; resource.slots.len()];
            for (slot_index, slot) in resource.slots.iter().enumerate() {
                old_to_new[slot_index] = bytes.len() / DISK_RECORD_BYTES + 1;
                let pages = if !slot.primary.editable {
                    vec![slot.original.clone()]
                } else {
                    let entry_index = *entry_by_slot
                        .get(&(resource_index, slot_index))
                        .ok_or_else(|| "可编辑 DOLL 页缺少 translation.json 索引".to_owned())?;
                    let translation = translations
                        .get(entry_index)
                        .ok_or_else(|| "translation.json 条目数在编码时发生变化".to_owned())?;
                    let original_message = slot
                        .primary
                        .message
                        .as_deref()
                        .ok_or_else(|| "可编辑 DOLL 页缺少原始 message".to_owned())?;
                    if translation.message == original_message {
                        vec![slot.original.clone()]
                    } else {
                        changed_records += 1;
                        let pages = encode_doll_pages(
                            slot,
                            &parsed_translations[entry_index],
                            &font_build,
                        )?;
                        if pages.len() > 1 && slot_index < DOLL_FIXED_REFERENCE_RECORDS {
                            return Err(format!(
                                "DOLL 记录 {} 不能扩页：RIO53 将 1–22 和 288 硬编码为记录号；从第 {} 条之后开始扩页才不会改变这些读取路径",
                                slot_index + 1,
                                DOLL_FIXED_REFERENCE_RECORDS + 1
                            ));
                        }
                        pages
                    }
                };
                if pages.is_empty() {
                    return Err(format!("DOLL slot {slot_index} 编码后没有记录"));
                }
                page_counts[slot_index] = pages.len();
                doll_pages_inserted += pages.len() - 1;
                for page in pages {
                    if page.len() != DISK_RECORD_BYTES {
                        return Err("DOLL 续页编码结果不是 256 字节".into());
                    }
                    bytes.extend_from_slice(&page);
                }
            }
            doll_old_to_new = Some(old_to_new);
            doll_page_counts = Some(page_counts);
            bytes
        } else {
            let mut bytes = resource.file.data.clone();
            for (slot_index, slot) in resource.slots.iter().enumerate() {
                if !slot.primary.editable {
                    continue;
                }
                let entry_index = *entry_by_slot
                    .get(&(resource_index, slot_index))
                    .ok_or_else(|| "可编辑文本槽缺少 translation.json 索引".to_owned())?;
                let translation = translations
                    .get(entry_index)
                    .ok_or_else(|| "translation.json 条目数在编码时发生变化".to_owned())?;
                let original_message = slot
                    .primary
                    .message
                    .as_deref()
                    .ok_or_else(|| "可编辑文本槽缺少原始 message".to_owned())?;
                if translation.message == original_message {
                    continue;
                }
                let encoded =
                    encode_fixed_text_slot(slot, &parsed_translations[entry_index], &font_build)?;
                let destination_end = slot
                    .primary
                    .offset
                    .checked_add(slot.primary.capacity)
                    .ok_or_else(|| "文本槽目标偏移溢出".to_owned())?;
                let destination = bytes
                    .get_mut(slot.primary.offset..destination_end)
                    .ok_or_else(|| "文本槽目标范围越过资源文件".to_owned())?;
                destination.copy_from_slice(&encoded);
                changed_records += 1;
            }
            bytes
        };
        if replacement.len() % DISK_RECORD_BYTES != 0 {
            return Err(format!(
                "{} 替换文件长度不是扇区整数倍",
                resource.file.display_name
            ));
        }
        expected_files[resource.disk_index]
            .insert(resource.file.directory_index, replacement.clone());
        disk_replacements[resource.disk_index].push(n88::FileReplacement {
            directory_index: resource.file.directory_index,
            data: replacement.clone(),
        });
        replacement_resources.push(replacement);
    }

    let old_to_new = doll_old_to_new
        .as_ref()
        .ok_or_else(|| "缺少 DOLL 记录映射".to_owned())?;
    if doll_pages_inserted != 0 {
        let disk_index = catalog
            .resources
            .iter()
            .find(|resource| resource.primary.file_name == "DOLL")
            .map(|resource| resource.disk_index)
            .ok_or_else(|| "缺少 DOLL 磁盘索引".to_owned())?;
        let des2 = prepared.disks[disk_index]
            .parsed
            .files
            .iter()
            .find(|file| file.display_name.eq_ignore_ascii_case("DES2"))
            .ok_or_else(|| "DOLL 扩页需要 DES2，但 disk_b 中缺少 DES2".to_owned())?;
        let (patched, reference_count) = patch_doll_references(&des2.data, old_to_new)?;
        if reference_count == 0 {
            return Err("DOLL 已扩页，但 DES2 中没有可重映射的 M#### 引用".into());
        }
        expected_files[disk_index].insert(des2.directory_index, patched.clone());
        disk_replacements[disk_index].push(n88::FileReplacement {
            directory_index: des2.directory_index,
            data: patched,
        });
    }

    let mut rebuilt_disks = prepared
        .disks
        .iter()
        .map(|disk| disk.source_snapshot.clone())
        .collect::<Vec<_>>();
    for (disk_index, replacements) in disk_replacements.iter().enumerate() {
        if !replacements.is_empty() {
            rebuilt_disks[disk_index] =
                n88::rebuild_file_data(&rebuilt_disks[disk_index], replacements).map_err(
                    |error| {
                        format!(
                            "重建 disk_{} 文件簇链失败：{error}",
                            prepared.disks[disk_index].label
                        )
                    },
                )?;
        }
    }

    verify_rebuilt_disks(
        prepared,
        catalog,
        &rebuilt_disks,
        &expected_files,
        &parsed_translations,
        &font_build.plan.mapping_used,
        old_to_new,
        doll_page_counts
            .as_deref()
            .ok_or_else(|| "缺少 DOLL 页数映射".to_owned())?,
    )?;
    let outer_d88 = rebuilt_disks.iter().flatten().copied().collect::<Vec<_>>();
    let unpack_plan = crate::inspect_container(&outer_d88)?;
    let rebuilt_container_sha256 = unpack_plan.source_sha256.clone();
    let font_report = font_build.report.clone();
    let report = TextBuildReport {
        source_container_sha256: prepared.source_manifest.source_sha256.clone(),
        rebuilt_container_sha256,
        changed_records,
        total_records: catalog.entries.len(),
        doll_pages_inserted,
        resource_lengths: catalog
            .resources
            .iter()
            .enumerate()
            .map(|(index, resource)| {
                (
                    format!(
                        "disk_{}/{}",
                        resource.primary.disk_label, resource.primary.file_name
                    ),
                    replacement_resources[index].len(),
                )
            })
            .collect(),
        font: font_report,
    };

    Ok(TextBuild {
        outer_d88,
        disk_bytes: rebuilt_disks,
        kanji1_rom: font_build.rom.clone(),
        font: font_build,
        report,
    })
}

fn encode_fixed_text_slot(
    slot: &TextSlot,
    message: &ParsedTranslation,
    font_build: &Pc88FontBuild,
) -> Result<Vec<u8>> {
    if message.control_sequence != visible_control_sequence(slot) {
        return Err(format!(
            "文本槽 {} 控制码序列与原文不一致",
            slot.primary.index
        ));
    }
    let mut output = Vec::with_capacity(slot.primary.capacity);
    output.extend_from_slice(&slot.original[..ESC_K.len()]);
    for part in &message.parts {
        match part {
            MessagePart::Text(text) => output.extend_from_slice(&font::encode_display_run(
                &font_build.resources,
                &font_build.plan,
                text,
            )?),
            MessagePart::Control(operation) => output.extend_from_slice(&encode_control(operation)),
        }
    }
    for operation in &slot.preserved_suffix {
        output.extend_from_slice(&encode_control(operation));
    }
    if slot.post_next_page_e {
        output.extend_from_slice(&[0x23, 0x65]);
    }

    let fill_end = slot.primary.capacity - usize::from(slot.final_nul);
    if output.len() > fill_end {
        return Err(format!(
            "{} slot {} 注入需要 {} 字节，读取器固定字段仅容纳 {} 字节",
            slot.primary.index,
            slot.primary.message.as_deref().unwrap_or("opaque"),
            output.len(),
            fill_end
        ));
    }
    output.resize(fill_end, 0x20);
    if slot.final_nul {
        output.push(0);
    }
    if output.len() != slot.primary.capacity {
        return Err("固定文本槽写入长度不一致".into());
    }
    Ok(output)
}

fn encode_doll_pages(
    slot: &TextSlot,
    message: &ParsedTranslation,
    font_build: &Pc88FontBuild,
) -> Result<Vec<Vec<u8>>> {
    if !slot
        .preserved_suffix
        .last()
        .is_some_and(TextOperation::is_terminal)
    {
        return Err(format!(
            "DOLL slot {} 缺少保留的 e/n 结束操作",
            slot.primary.index
        ));
    }
    if !slot.final_nul || slot.primary.capacity != DISK_RECORD_BYTES {
        return Err(format!(
            "DOLL slot {} 不是 256 字节 NUL 结尾记录",
            slot.primary.index
        ));
    }

    let mut atoms = Vec::<Vec<u8>>::new();
    for part in &message.parts {
        match part {
            MessagePart::Text(text) => {
                let encoded =
                    font::encode_display_run(&font_build.resources, &font_build.plan, text)?;
                if encoded.len() % 2 != 0 {
                    return Err("PC-88 JIS 编码产生奇数字节文本".into());
                }
                atoms.extend(encoded.chunks_exact(2).map(<[u8]>::to_vec));
            }
            MessagePart::Control(operation) => atoms.push(encode_control(operation)),
        }
    }

    let post_terminal_bytes = if slot.post_next_page_e { 2 } else { 0 };
    let suffix_bytes = slot
        .preserved_suffix
        .iter()
        .map(|operation| encode_control(operation).len())
        .sum::<usize>();
    let max_body_bytes = (slot.primary.capacity
        - ESC_K.len()
        - suffix_bytes
        - usize::from(slot.final_nul)
        - post_terminal_bytes)
        & !1;
    if max_body_bytes < 10 {
        return Err("DOLL 记录容量不足以容纳文本和操作码".into());
    }

    let mut bodies = vec![Vec::<u8>::new()];
    for atom in atoms {
        if atom.len() > max_body_bytes {
            return Err("RIO53 操作码超过单页可用空间".into());
        }
        let current = bodies
            .last_mut()
            .ok_or_else(|| "DOLL 页缓冲为空".to_owned())?;
        if current.len() + atom.len() > max_body_bytes {
            bodies.push(Vec::new());
        }
        bodies
            .last_mut()
            .ok_or_else(|| "DOLL 页缓冲为空".to_owned())?
            .extend_from_slice(&atom);
    }

    let total_pages = bodies.len();
    let mut pages = Vec::with_capacity(total_pages);
    for (page_index, body) in bodies.into_iter().enumerate() {
        let is_final = page_index + 1 == total_pages;
        let mut output = Vec::with_capacity(slot.primary.capacity);
        output.extend_from_slice(&ESC_K);
        output.extend_from_slice(&body);
        if is_final {
            for operation in &slot.preserved_suffix {
                output.extend_from_slice(&encode_control(operation));
            }
        } else {
            output.extend_from_slice(&encode_control(&TextOperation::NextPage));
        }
        if is_final && slot.post_next_page_e {
            output.extend_from_slice(&[0x23, 0x65]);
        }
        let fill_end = slot.primary.capacity - 1;
        if output.len() > fill_end {
            return Err(format!(
                "DOLL slot {} 的分页编码超出 256 字节",
                slot.primary.index
            ));
        }
        output.resize(fill_end, 0x20);
        output.push(0);
        pages.push(output);
    }
    Ok(pages)
}

fn encode_control(operation: &TextOperation) -> Vec<u8> {
    fn pair(code: u16, output: &mut Vec<u8>) {
        output.extend_from_slice(&code.to_be_bytes());
    }
    fn digit(value: u8, output: &mut Vec<u8>) {
        pair(0x2330 + u16::from(value), output);
    }
    let mut output = Vec::with_capacity(10);
    match operation {
        TextOperation::Row => pair(JIS_R, &mut output),
        TextOperation::End => pair(JIS_E, &mut output),
        TextOperation::NextPage => pair(JIS_N, &mut output),
        TextOperation::Graphic { id } => {
            pair(JIS_C, &mut output);
            digit(id / 10, &mut output);
            digit(id % 10, &mut output);
        }
        TextOperation::Variable { index, op, value } => {
            pair(JIS_I, &mut output);
            digit(index / 10, &mut output);
            digit(index % 10, &mut output);
            pair(
                match op {
                    VariableOperation::Add => JIS_PLUS,
                    VariableOperation::Subtract => JIS_MINUS,
                    VariableOperation::Assign => JIS_EQUALS,
                },
                &mut output,
            );
            digit(*value, &mut output);
        }
        TextOperation::RawPair { value } => pair(*value, &mut output),
    }
    output
}

fn patch_doll_references(source: &[u8], old_to_new: &[usize]) -> Result<(Vec<u8>, usize)> {
    let mut output = source.to_vec();
    let mut cursor = 0usize;
    let mut references = 0usize;
    while cursor + 5 <= output.len() {
        if output[cursor] == b'M'
            && output[cursor + 1..cursor + 5]
                .iter()
                .all(u8::is_ascii_digit)
        {
            let old_index = output[cursor + 1..cursor + 5]
                .iter()
                .fold(0usize, |value, digit| {
                    value * 10 + usize::from(*digit - b'0')
                });
            if old_index == 0 || old_index > old_to_new.len() {
                return Err(format!("DES2 中的 M{old_index:04} 超出 DOLL 原记录范围"));
            }
            let new_index = old_to_new[old_index - 1];
            if new_index == 0 || new_index > 9999 {
                return Err(format!("DOLL 新记录号 {new_index} 不能编码为 M####"));
            }
            output[cursor + 1..cursor + 5].copy_from_slice(format!("{new_index:04}").as_bytes());
            references += 1;
            cursor += 5;
        } else {
            cursor += 1;
        }
    }
    Ok((output, references))
}

fn parse_text_resource_slots(
    disk_label: &str,
    file_name: &str,
    data: &[u8],
    layout: &str,
) -> Result<Vec<TextSlot>> {
    let slot_layout = match layout {
        "fixed_256" => TextResourceLayout::Fixed256 { final_nul: true },
        "doll_256_byte_pages" => TextResourceLayout::DollPages,
        "base_3x85_in_256" => TextResourceLayout::BaseThreeFields,
        _ => return Err(format!("未知文本资源布局 {layout:?}")),
    };
    let ranges = slot_ranges(slot_layout, data.len(), file_name)?;
    let mut slots = Vec::with_capacity(ranges.len());
    for (index, (offset, capacity)) in ranges.into_iter().enumerate() {
        let end = offset + capacity;
        let original = data
            .get(offset..end)
            .ok_or_else(|| format!("{file_name} slot {index} 越过重建资源末尾"))?
            .to_vec();
        slots.push(parse_text_slot(
            disk_label,
            file_name,
            index,
            offset,
            original,
            matches!(
                slot_layout,
                TextResourceLayout::Fixed256 { final_nul: true } | TextResourceLayout::DollPages
            ),
            file_name == "DOLL",
        )?);
    }
    Ok(slots)
}

fn restore_dynamic_targets(message: &str, mapping_used: &[vn_font::font_88::MappingUse]) -> String {
    let reverse_mapping = mapping_used
        .iter()
        .map(|mapping| (mapping.carrier, mapping.target))
        .collect::<BTreeMap<_, _>>();
    message
        .chars()
        .map(|character| {
            reverse_mapping
                .get(&character)
                .copied()
                .unwrap_or(character)
        })
        .collect()
}

fn verify_rebuilt_disks(
    prepared: &crate::PreparedResources,
    catalog: &TextCatalog,
    rebuilt_disks: &[Vec<u8>],
    expected_files: &[BTreeMap<usize, Vec<u8>>],
    translations: &[ParsedTranslation],
    mapping_used: &[vn_font::font_88::MappingUse],
    doll_old_to_new: &[usize],
    doll_page_counts: &[usize],
) -> Result<()> {
    if rebuilt_disks.len() != prepared.disks.len() || expected_files.len() != prepared.disks.len() {
        return Err("重建盘数与源盘数不一致".into());
    }
    for (disk_index, disk_bytes) in rebuilt_disks.iter().enumerate() {
        let rebuilt = n88::parse_disk(disk_bytes).map_err(|error| {
            format!(
                "重建后的 disk_{} 无法重解析：{error}",
                prepared.disks[disk_index].label
            )
        })?;
        let original = &prepared.disks[disk_index].parsed;
        if rebuilt.files.len() != original.files.len() {
            return Err(format!(
                "重建后的 disk_{} 文件数变化",
                prepared.disks[disk_index].label
            ));
        }
        for original_file in &original.files {
            let rebuilt_file = rebuilt
                .files
                .iter()
                .find(|file| file.directory_index == original_file.directory_index)
                .ok_or_else(|| {
                    format!(
                        "disk_{} 丢失目录项 {}",
                        prepared.disks[disk_index].label, original_file.directory_index
                    )
                })?;
            let expected = expected_files[disk_index]
                .get(&original_file.directory_index)
                .map(Vec::as_slice)
                .unwrap_or(original_file.data.as_slice());
            if rebuilt_file.data != expected {
                return Err(format!(
                    "disk_{} 文件 {} 与重建前/预期文本资源不一致",
                    prepared.disks[disk_index].label, original_file.display_name
                ));
            }
        }
    }

    for (resource_index, resource) in catalog.resources.iter().enumerate() {
        let data = expected_files
            .get(resource.disk_index)
            .and_then(|files| files.get(&resource.file.directory_index))
            .ok_or_else(|| "文本资源替换列表不完整".to_owned())?;
        let _parsed = parse_text_resource_slots(
            &resource.primary.disk_label,
            &resource.primary.file_name,
            data,
            resource.primary.layout.as_str(),
        )?;
        if resource.primary.file_name == "DOLL" {
            if doll_old_to_new.len() != resource.slots.len()
                || doll_page_counts.len() != resource.slots.len()
            {
                return Err("DOLL 旧记录映射与原资源记录数不一致".into());
            }
            for (old_slot_index, old_slot) in resource.slots.iter().enumerate() {
                if !old_slot.primary.editable {
                    if doll_page_counts[old_slot_index] != 1 {
                        return Err(format!("不透明 DOLL slot {old_slot_index} 被拆页"));
                    }
                    continue;
                }
                let expected_entry_index = catalog
                    .entry_slots
                    .iter()
                    .position(|key| *key == (resource_index, old_slot_index))
                    .ok_or_else(|| "DOLL slot 缺少翻译索引".to_owned())?;
                let first_page = doll_old_to_new[old_slot_index]
                    .checked_sub(1)
                    .ok_or_else(|| "DOLL 新记录号为零".to_owned())?;
                let page_count = doll_page_counts[old_slot_index];
                let continuation_suffix = [TextOperation::NextPage];
                let mut actual_message = String::new();
                for page_offset in 0..page_count {
                    let page_index = first_page + page_offset;
                    let start = page_index * DISK_RECORD_BYTES;
                    let end = start + DISK_RECORD_BYTES;
                    let raw_page = data
                        .get(start..end)
                        .ok_or_else(|| format!("DOLL 新续页 {page_index} 越界"))?
                        .to_vec();
                    let suffix_hint = if page_offset + 1 < page_count {
                        continuation_suffix.as_slice()
                    } else {
                        old_slot.preserved_suffix.as_slice()
                    };
                    let page = parse_text_slot_with_suffix(
                        &resource.primary.disk_label,
                        "DOLL",
                        page_index,
                        start,
                        raw_page,
                        true,
                        true,
                        Some(suffix_hint),
                    )?;
                    let message = page
                        .primary
                        .message
                        .as_deref()
                        .ok_or_else(|| format!("DOLL 新续页 {page_index} 无法解析"))?
                        .to_owned();
                    if page_offset + 1 < page_count {
                        if page.preserved_suffix != continuation_suffix || page.post_next_page_e {
                            return Err(format!("DOLL 新续页 {page_index} 没有 n 操作"));
                        }
                    } else if page.preserved_suffix != old_slot.preserved_suffix
                        || page.post_next_page_e != old_slot.post_next_page_e
                    {
                        return Err(format!("DOLL 新续页 {page_index} 固定结束码与原记录不一致"));
                    }
                    actual_message.push_str(&restore_dynamic_targets(&message, mapping_used));
                }
                let expected_message = translations
                    .get(expected_entry_index)
                    .map(|message| message.canonical_message.as_str())
                    .ok_or_else(|| "DOLL 译文重解析索引越界".to_owned())?;
                if actual_message != expected_message {
                    return Err(format!(
                        "重建后的 disk_{}/DOLL 原 slot {} 续页与译文不一致",
                        resource.primary.disk_label, old_slot_index
                    ));
                }
            }
        } else {
            for (slot_index, slot) in resource.slots.iter().enumerate() {
                if !slot.primary.editable {
                    continue;
                }
                let expected_entry_index = catalog
                    .entry_slots
                    .iter()
                    .position(|key| *key == (resource_index, slot_index))
                    .ok_or_else(|| "文本 slot 缺少翻译索引".to_owned())?;
                let start = slot.primary.offset;
                let end = start + slot.primary.capacity;
                let raw_slot = data
                    .get(start..end)
                    .ok_or_else(|| "重解析的文本 slot 越过资源末尾".to_owned())?
                    .to_vec();
                let rebuilt_slot = parse_text_slot_with_suffix(
                    &resource.primary.disk_label,
                    &resource.primary.file_name,
                    slot_index,
                    start,
                    raw_slot,
                    slot.final_nul,
                    false,
                    Some(&slot.preserved_suffix),
                )?;
                if rebuilt_slot.post_next_page_e != slot.post_next_page_e {
                    return Err(format!("文本 slot {} 的固定结束码变化", slot_index));
                }
                let actual = restore_dynamic_targets(
                    rebuilt_slot
                        .primary
                        .message
                        .as_deref()
                        .ok_or_else(|| "重解析的文本 slot 不可读".to_owned())?,
                    mapping_used,
                );
                let expected_message = translations
                    .get(expected_entry_index)
                    .map(|message| message.canonical_message.as_str())
                    .ok_or_else(|| "译文重解析索引越界".to_owned())?;
                if actual != expected_message {
                    return Err(format!(
                        "重建后的 disk_{}/{} slot {} 与译文不一致",
                        resource.primary.disk_label, resource.primary.file_name, slot_index
                    ));
                }
            }
        }
    }
    Ok(())
}

fn build_output_files(
    build: &TextBuild,
    primary_bytes: &[u8],
    translation_bytes: &[u8],
) -> vn_cli::Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = BTreeMap::new();
    files.insert(PathBuf::from("rebuilt.d88"), build.outer_d88.clone());
    files.insert(PathBuf::from("KANJI1.ROM"), build.kanji1_rom.clone());
    for (index, label) in ["a", "b", "c"].into_iter().enumerate() {
        let disk = build
            .disk_bytes
            .get(index)
            .ok_or_else(|| vn_cli::Error("重建结果缺少 A/B/C 盘".into()))?;
        files.insert(PathBuf::from(format!("disk_{label}.d88")), disk.clone());
    }
    files.insert(PathBuf::from("primary.json"), primary_bytes.to_vec());
    files.insert(
        PathBuf::from("translation.json"),
        translation_bytes.to_vec(),
    );
    let report_bytes = json_bytes(&build.report).map_err(vn_cli::Error::from)?;
    files.insert(PathBuf::from("build_report.json"), report_bytes);
    let font_manifest = json_bytes(&build.font.report.manifest).map_err(vn_cli::Error::from)?;
    files.insert(PathBuf::from("font_manifest.json"), font_manifest);
    let mapping = json_bytes(&build.font.plan.mapping_used).map_err(vn_cli::Error::from)?;
    files.insert(PathBuf::from("mapping_used.json"), mapping);
    if let Some(glyph_table) = &build.font.report.generated_glyph_table {
        files.insert(PathBuf::from("generated_glyphs.fcg1"), glyph_table.clone());
    }
    let unpack_plan = crate::inspect_container(&build.outer_d88).map_err(vn_cli::Error::from)?;
    let manifest = serde_json::json!({
        "format": "executive-road-pc88-d88-split",
        "version": 1,
        "source_length": unpack_plan.source_length,
        "source_sha256": unpack_plan.source_sha256,
        "disks": unpack_plan.disks,
        "warnings": unpack_plan.warnings,
    });
    files.insert(
        PathBuf::from("manifest.json"),
        json_bytes(&manifest).map_err(vn_cli::Error::from)?,
    );
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fullwidth_operator_letters_remain_text_with_escape_controls() {
        let parsed = parse_translation_message("咖啡馆「Ｌｉｍｅ」\\n对话\\c18")
            .expect("parse escaped controls alongside fullwidth Latin text");
        assert_eq!(
            parsed.control_sequence,
            [TextOperation::Row, TextOperation::Graphic { id: 18 }]
        );

        let build = font::build_pc88_font(&[], &parsed.display_texts)
            .expect("plan carriers for fullwidth operator letters");
        let encoded = font::encode_display_run(&build.resources, &build.plan, "Ｌｉｍｅ")
            .expect("encode fullwidth Latin text through PC-88 carriers");
        assert_eq!(encoded.len() % 2, 0);
        assert!(encoded
            .chunks_exact(2)
            .all(|pair| !font::is_reserved_text_op_jis(u16::from_be_bytes([pair[0], pair[1]]))));
    }
}
