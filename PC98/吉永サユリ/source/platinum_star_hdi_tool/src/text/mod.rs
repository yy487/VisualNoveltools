mod controls;
mod encoding;
mod exe_text;
mod gdm;
mod model;
mod pklite;

use encoding::{characters_needing_carriers, native_double_byte_slots};
use exe_text::{extract_spd, extract_spg, inject_spd, inject_spg};
use gdm::{extract_scenario, inject_scenario};
use model::{
    ExportManifest, ExportPolicy, ExportTotals, ExportedFile, FontManifest, FontMappingDocument,
    ImportManifest, ImportedFile, TextDocument,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use vn_font::font_98::{self, EncodingPlan, SubstitutionMap, EMBEDDED_FONT, FONT_FACE};

pub const TEXT_MANIFEST_FILENAME: &str = ".platinum_star_text_manifest.json";
pub const TEXT_MANIFEST_FORMAT: &str = "platinum-star-text-export-v1";
pub const TEXT_IMPORT_MANIFEST_FILENAME: &str = ".platinum_star_import_manifest.json";
pub const TEXT_IMPORT_MANIFEST_FORMAT: &str = "platinum-star-text-import-v1";

pub struct TextExportInfo {
    pub source_files: usize,
    pub scenario_blocks: usize,
    pub scenario_entries: usize,
    pub spd_entries: usize,
    pub spg_entries: usize,
}

pub struct TextExportReport {
    pub output_root: PathBuf,
    pub manifest: PathBuf,
    pub source_files: usize,
    pub scenario_blocks: usize,
    pub entries: usize,
    pub warnings: Vec<String>,
}

pub struct PreparedTextExport {
    source_root: PathBuf,
    output: PathBuf,
    overwrite: bool,
    documents: Vec<(&'static str, TextDocument)>,
    manifest: ExportManifest,
    info: TextExportInfo,
    warnings: Vec<String>,
}

pub struct TextImportInfo {
    pub source_files: usize,
    pub scenario_entries: usize,
    pub spd_entries: usize,
    pub spg_entries: usize,
    pub changed_entries: usize,
}

pub struct TextImportReport {
    pub output_root: PathBuf,
    pub manifest: PathBuf,
    pub copied_files: usize,
    pub changed_entries: usize,
    pub warnings: Vec<String>,
}

struct SnapshotFile {
    relative: PathBuf,
    bytes: Vec<u8>,
}

struct FontArtifact {
    relative: PathBuf,
    bytes: Vec<u8>,
    mapping: FontMappingDocument,
    patched_glyphs: usize,
    reserved_cp932_slots: usize,
}

pub struct PreparedTextImport {
    source_root: PathBuf,
    translation_root: PathBuf,
    output: PathBuf,
    overwrite: bool,
    directories: Vec<PathBuf>,
    files: Vec<SnapshotFile>,
    replacements: BTreeMap<PathBuf, Vec<u8>>,
    font: Option<FontArtifact>,
    manifest: ImportManifest,
    info: TextImportInfo,
    warnings: Vec<String>,
}

impl PreparedTextExport {
    pub fn source_root(&self) -> &Path {
        &self.source_root
    }

    pub fn output(&self) -> &Path {
        &self.output
    }

    pub fn inspection(&self) -> &TextExportInfo {
        &self.info
    }

    pub fn execute(self) -> Result<TextExportReport, String> {
        let parent = self.output.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建输出父目录 {}: {error}", parent.display()))?;
        let stem = self
            .output
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("输出目录名不是有效 Unicode: {}", self.output.display()))?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("系统时钟错误: {error}"))?
            .as_nanos();
        let stage = parent.join(format!(
            ".{stem}.platinum-star-text.tmp-{}-{nonce}",
            std::process::id()
        ));
        let backup = parent.join(format!(
            ".{stem}.platinum-star-text.backup-{}-{nonce}",
            std::process::id()
        ));
        validate_sibling(&self.output, &stage)?;
        validate_sibling(&self.output, &backup)?;
        fs::create_dir(&stage)
            .map_err(|error| format!("无法创建临时输出目录 {}: {error}", stage.display()))?;

        let write_result = (|| {
            for (name, document) in &self.documents {
                write_json(&stage.join(name), document)?;
            }
            write_json(&stage.join(TEXT_MANIFEST_FILENAME), &self.manifest)?;
            for (name, _) in &self.documents {
                verify_json(&stage.join(name))?;
            }
            verify_json(&stage.join(TEXT_MANIFEST_FILENAME))?;
            Ok::<(), String>(())
        })();
        if let Err(error) = write_result {
            let _ = safe_remove_dir(&self.output, &stage);
            return Err(error);
        }

        if self.output.exists() {
            if !self.overwrite {
                let _ = safe_remove_dir(&self.output, &stage);
                return Err(format!(
                    "输出已存在；需要显式启用覆盖: {}",
                    self.output.display()
                ));
            }
            validate_managed_output(&self.output)?;
            if let Err(error) = fs::rename(&self.output, &backup) {
                let _ = safe_remove_dir(&self.output, &stage);
                return Err(format!(
                    "无法把旧输出移到事务备份 {}: {error}",
                    backup.display()
                ));
            }
            if let Err(error) = fs::rename(&stage, &self.output) {
                let rollback = fs::rename(&backup, &self.output);
                let _ = safe_remove_dir(&self.output, &stage);
                return Err(format!(
                    "无法提交新输出: {error}；回滚旧输出{}",
                    if rollback.is_ok() { "成功" } else { "失败" }
                ));
            }
            safe_remove_dir(&self.output, &backup)?;
        } else {
            fs::rename(&stage, &self.output)
                .map_err(|error| format!("无法提交输出目录 {}: {error}", self.output.display()))?;
        }

        let entries = self.info.scenario_entries + self.info.spd_entries + self.info.spg_entries;
        Ok(TextExportReport {
            manifest: self.output.join(TEXT_MANIFEST_FILENAME),
            output_root: self.output,
            source_files: self.info.source_files,
            scenario_blocks: self.info.scenario_blocks,
            entries,
            warnings: self.warnings,
        })
    }
}

impl PreparedTextImport {
    pub fn source_root(&self) -> &Path {
        &self.source_root
    }

    pub fn translation_root(&self) -> &Path {
        &self.translation_root
    }

    pub fn output(&self) -> &Path {
        &self.output
    }

    pub fn inspection(&self) -> &TextImportInfo {
        &self.info
    }

    pub fn execute(self) -> Result<TextImportReport, String> {
        let parent = self.output.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建输出父目录 {}: {error}", parent.display()))?;
        let stem = self
            .output
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("输出目录名不是有效 Unicode: {}", self.output.display()))?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("系统时钟错误: {error}"))?
            .as_nanos();
        let stage = parent.join(format!(
            ".{stem}.platinum-star-import.tmp-{}-{nonce}",
            std::process::id()
        ));
        let backup = parent.join(format!(
            ".{stem}.platinum-star-import.backup-{}-{nonce}",
            std::process::id()
        ));
        validate_import_sibling(&self.output, &stage)?;
        validate_import_sibling(&self.output, &backup)?;
        fs::create_dir(&stage)
            .map_err(|error| format!("无法创建临时输出目录 {}: {error}", stage.display()))?;

        let write_result = (|| {
            for relative in &self.directories {
                fs::create_dir_all(stage.join(relative)).map_err(|error| {
                    format!("无法创建输出子目录 {}: {error}", relative.display())
                })?;
            }
            for source_file in &self.files {
                let bytes = self
                    .replacements
                    .get(&source_file.relative)
                    .unwrap_or(&source_file.bytes);
                write_bytes(&stage.join(&source_file.relative), bytes)?;
            }
            write_json(&stage.join(TEXT_IMPORT_MANIFEST_FILENAME), &self.manifest)?;
            verify_json(&stage.join(TEXT_IMPORT_MANIFEST_FILENAME))?;
            for source_file in &self.files {
                let expected = self
                    .replacements
                    .get(&source_file.relative)
                    .unwrap_or(&source_file.bytes);
                let actual = fs::read(stage.join(&source_file.relative)).map_err(|error| {
                    format!(
                        "无法复核输出文件 {}: {error}",
                        source_file.relative.display()
                    )
                })?;
                if actual != *expected {
                    return Err(format!(
                        "写出后内容不一致: {}",
                        source_file.relative.display()
                    ));
                }
            }
            if let Some(font) = &self.font {
                write_bytes(&stage.join(&font.relative), &font.bytes)?;
                write_json(&stage.join("font_mapping.json"), &font.mapping)?;
                verify_json(&stage.join("font_mapping.json"))?;
            }
            Ok::<(), String>(())
        })();
        if let Err(error) = write_result {
            let _ = safe_remove_import_dir(&self.output, &stage);
            return Err(error);
        }

        if self.output.exists() {
            if !self.overwrite {
                let _ = safe_remove_import_dir(&self.output, &stage);
                return Err(format!(
                    "输出已存在；需要显式启用覆盖: {}",
                    self.output.display()
                ));
            }
            validate_managed_import_output(&self.output)?;
            if let Err(error) = fs::rename(&self.output, &backup) {
                let _ = safe_remove_import_dir(&self.output, &stage);
                return Err(format!(
                    "无法把旧输出移到事务备份 {}: {error}",
                    backup.display()
                ));
            }
            if let Err(error) = fs::rename(&stage, &self.output) {
                let rollback = fs::rename(&backup, &self.output);
                let _ = safe_remove_import_dir(&self.output, &stage);
                return Err(format!(
                    "无法提交新输出: {error}；回滚旧输出{}",
                    if rollback.is_ok() { "成功" } else { "失败" }
                ));
            }
            safe_remove_import_dir(&self.output, &backup)?;
        } else {
            fs::rename(&stage, &self.output)
                .map_err(|error| format!("无法提交输出目录 {}: {error}", self.output.display()))?;
        }

        Ok(TextImportReport {
            manifest: self.output.join(TEXT_IMPORT_MANIFEST_FILENAME),
            output_root: self.output,
            copied_files: self.info.source_files,
            changed_entries: self.info.changed_entries,
            warnings: self.warnings,
        })
    }
}

pub fn prepare_text_export(
    source_root: &Path,
    output: &Path,
    jobs: usize,
    overwrite: bool,
) -> Result<PreparedTextExport, String> {
    if jobs == 0 || jobs > 64 {
        return Err("并行任务数 jobs 必须在 1..64 之间".into());
    }
    if !source_root.is_dir() {
        return Err(format!(
            "文本来源必须是含 SCENARIO.GDM、SPD.BIN、SPG.BIN 的目录: {}",
            source_root.display()
        ));
    }
    if output.exists() {
        if !overwrite {
            return Err(format!("输出已存在；默认不覆盖: {}", output.display()));
        }
        validate_managed_output(output)?;
    }

    let scenario_path = source_root.join("SCENARIO.GDM");
    let spd_path = source_root.join("SPD.BIN");
    let spg_path = source_root.join("SPG.BIN");
    let scenario_bytes = read_source(&scenario_path)?;
    let spd_bytes = read_source(&spd_path)?;
    let spg_bytes = read_source(&spg_path)?;

    let (scenario, spd, spg) = if jobs >= 3 {
        std::thread::scope(|scope| -> Result<_, String> {
            let scenario_job = scope.spawn(|| extract_scenario(&scenario_bytes, jobs - 2));
            let spd_job = scope.spawn(|| extract_spd(&spd_bytes));
            let spg_job = scope.spawn(|| extract_spg(&spg_bytes));
            Ok((
                scenario_job
                    .join()
                    .map_err(|_| "SCENARIO.GDM 提取线程异常退出".to_string())?,
                spd_job
                    .join()
                    .map_err(|_| "SPD.BIN 提取线程异常退出".to_string())?,
                spg_job
                    .join()
                    .map_err(|_| "SPG.BIN 提取线程异常退出".to_string())?,
            ))
        })?
    } else {
        (
            extract_scenario(&scenario_bytes, jobs),
            extract_spd(&spd_bytes),
            extract_spg(&spg_bytes),
        )
    };
    let scenario = scenario?;
    let spd = spd?;
    let spg = spg?;

    let source_label = source_root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("source")
        .to_string();
    let exported = vec![
        exported_file("SCENARIO.GDM", "scenario.gdm.json", &scenario.document),
        exported_file("SPD.BIN", "spd.bin.json", &spd),
        exported_file("SPG.BIN", "spg.bin.json", &spg),
    ];
    let scenario_entries = scenario.document.entries.len();
    let spd_entries = spd.entries.len();
    let spg_entries = spg.entries.len();
    let info = TextExportInfo {
        source_files: 3,
        scenario_blocks: scenario.blocks,
        scenario_entries,
        spd_entries,
        spg_entries,
    };
    let manifest = ExportManifest {
        _format: TEXT_MANIFEST_FORMAT.into(),
        tool_version: env!("CARGO_PKG_VERSION").into(),
        source_root: source_label,
        files: exported,
        totals: ExportTotals {
            source_files: 3,
            entries: (scenario_entries + spd_entries + spg_entries) as u64,
            scenario_blocks: scenario.blocks as u64,
        },
        policy: ExportPolicy {
            editable_fields: vec!["name".into(), "message".into()],
            immutable_source_fields: vec!["_scr_name".into(), "scr_msg".into()],
            controls:
                "显示控制符从 message 中移除并保存在下划线元数据字段；回注时按原相对位置恢复。"
                    .into(),
            record_granularity:
                "每次独立发言、每个菜单项、每个独立帮助字符串各一条记录；不使用 message_parts。"
                    .into(),
        },
    };

    Ok(PreparedTextExport {
        source_root: source_root.to_path_buf(),
        output: output.to_path_buf(),
        overwrite,
        documents: vec![
            ("scenario.gdm.json", scenario.document),
            ("spd.bin.json", spd),
            ("spg.bin.json", spg),
        ],
        manifest,
        info,
        warnings: scenario.warnings,
    })
}

pub fn prepare_text_import(
    source_root: &Path,
    translation_root: &Path,
    output: &Path,
    jobs: usize,
    overwrite: bool,
) -> Result<PreparedTextImport, String> {
    if jobs == 0 || jobs > 64 {
        return Err("并行任务数 jobs 必须在 1..64 之间".into());
    }
    let source_root = fs::canonicalize(source_root)
        .map_err(|error| format!("无法解析游戏来源目录 {}: {error}", source_root.display()))?;
    let translation_root = fs::canonicalize(translation_root).map_err(|error| {
        format!(
            "无法解析翻译 JSON 目录 {}: {error}",
            translation_root.display()
        )
    })?;
    if !source_root.is_dir() {
        return Err(format!("游戏来源不是目录: {}", source_root.display()));
    }
    if !translation_root.is_dir() {
        return Err(format!(
            "翻译 JSON 来源不是目录: {}",
            translation_root.display()
        ));
    }
    let output = std::path::absolute(output)
        .map_err(|error| format!("无法解析输出路径 {}: {error}", output.display()))?;
    if output.starts_with(&source_root) || source_root.starts_with(&output) {
        return Err("输出目录不能与只读游戏来源目录互相包含".into());
    }
    if output.starts_with(&translation_root) || translation_root.starts_with(&output) {
        return Err("输出目录不能与只读翻译 JSON 目录互相包含".into());
    }
    if output.exists() {
        if !overwrite {
            return Err(format!("输出已存在；默认不覆盖: {}", output.display()));
        }
        validate_managed_import_output(&output)?;
    }

    validate_export_manifest(&translation_root)?;
    let (directories, files) = snapshot_source_tree(&source_root)?;
    let scenario_bytes = snapshot_file(&files, "SCENARIO.GDM")?;
    let spd_bytes = snapshot_file(&files, "SPD.BIN")?;
    let spg_bytes = snapshot_file(&files, "SPG.BIN")?;

    let (scenario, spd, spg) = if jobs >= 3 {
        std::thread::scope(|scope| -> Result<_, String> {
            let scenario_job = scope.spawn(|| extract_scenario(scenario_bytes, jobs - 2));
            let spd_job = scope.spawn(|| extract_spd(spd_bytes));
            let spg_job = scope.spawn(|| extract_spg(spg_bytes));
            Ok((
                scenario_job
                    .join()
                    .map_err(|_| "SCENARIO.GDM 回注预检线程异常退出".to_string())?,
                spd_job
                    .join()
                    .map_err(|_| "SPD.BIN 回注预检线程异常退出".to_string())?,
                spg_job
                    .join()
                    .map_err(|_| "SPG.BIN 回注预检线程异常退出".to_string())?,
            ))
        })?
    } else {
        (
            extract_scenario(scenario_bytes, jobs),
            extract_spd(spd_bytes),
            extract_spg(spg_bytes),
        )
    };
    let scenario = scenario?;
    let spd = spd?;
    let spg = spg?;

    let translated_scenario = read_document(&translation_root.join("scenario.gdm.json"))?;
    let translated_spd = read_document(&translation_root.join("spd.bin.json"))?;
    let translated_spg = read_document(&translation_root.join("spg.bin.json"))?;
    validate_translation_document(&scenario.document, &translated_scenario)?;
    validate_translation_document(&spd, &translated_spd)?;
    validate_translation_document(&spg, &translated_spg)?;

    let (font_plan, font) = prepare_font_artifact(
        &scenario.document,
        &translated_scenario,
        &spd,
        &translated_spd,
        &spg,
        &translated_spg,
        &files,
    )?;
    let (scenario_output, scenario_changed) = inject_scenario(
        scenario_bytes,
        &scenario.document,
        &translated_scenario,
        font_plan.as_ref(),
    )?;
    let (spd_output, spd_changed) =
        inject_spd(spd_bytes, &spd, &translated_spd, font_plan.as_ref())?;
    let (spg_output, spg_changed) =
        inject_spg(spg_bytes, &spg, &translated_spg, font_plan.as_ref())?;
    let changed_entries = scenario_changed + spd_changed + spg_changed;

    let mut warnings = scenario.warnings;
    if font.is_some() {
        warnings.push(
            "检测到译文包含 NP2 未加载字库页或 CP932 不可直接编码的字符；已按 vn-font 的 PC-98 NP2 计划生成 font.bmp 和映射清单。".into(),
        );
    }
    if spd_changed > 0 {
        warnings.push(
            "SPD.BIN 已生成解包后 MZ；未重新套 PKLITE 壳，尚需在目标运行环境验证加载。".into(),
        );
    }
    if spg_changed > 0 {
        warnings.push(
            "SPG.BIN 已生成解包后 MZ；未重新套 PKLITE 壳，尚需在目标运行环境验证加载。".into(),
        );
    }

    let output_files = vec![
        imported_file(
            "SCENARIO.GDM",
            scenario_bytes,
            &scenario_output,
            scenario_changed,
            if scenario_changed == 0 {
                "原始 GDM（未修改）"
            } else {
                "GDM（修改块使用字面量编码，未改块原样保留）"
            },
        ),
        imported_file(
            "SPD.BIN",
            spd_bytes,
            &spd_output,
            spd_changed,
            if spd_changed == 0 {
                "原始 PKLITE 文件（未修改）"
            } else {
                "解包后 MZ（未重新 PKLITE 压缩）"
            },
        ),
        imported_file(
            "SPG.BIN",
            spg_bytes,
            &spg_output,
            spg_changed,
            if spg_changed == 0 {
                "原始 PKLITE 文件（未修改）"
            } else {
                "解包后 MZ（未重新 PKLITE 压缩）"
            },
        ),
    ];
    let manifest = ImportManifest {
        _format: TEXT_IMPORT_MANIFEST_FORMAT.into(),
        tool_version: env!("CARGO_PKG_VERSION").into(),
        source_root: source_root.display().to_string(),
        translation_root: translation_root.display().to_string(),
        output_files,
        changed_entries: changed_entries as u64,
        warnings: warnings.clone(),
        font: font.as_ref().map(|artifact| FontManifest {
            base_font: "vn-font/assets/pc98/font.tmp".into(),
            base_sha256: sha256(EMBEDDED_FONT),
            output: artifact.relative.display().to_string(),
            output_sha256: sha256(&artifact.bytes),
            mapping: "font_mapping.json".into(),
            patched_glyphs: artifact.patched_glyphs as u64,
            reserved_cp932_slots: artifact.reserved_cp932_slots as u64,
            face: FONT_FACE.into(),
        }),
    };
    let mut replacements = BTreeMap::new();
    replacements.insert(PathBuf::from("SCENARIO.GDM"), scenario_output);
    replacements.insert(PathBuf::from("SPD.BIN"), spd_output);
    replacements.insert(PathBuf::from("SPG.BIN"), spg_output);
    let info = TextImportInfo {
        source_files: files.len(),
        scenario_entries: scenario.document.entries.len(),
        spd_entries: spd.entries.len(),
        spg_entries: spg.entries.len(),
        changed_entries,
    };

    Ok(PreparedTextImport {
        source_root,
        translation_root,
        output,
        overwrite,
        directories,
        files,
        replacements,
        font,
        manifest,
        info,
        warnings,
    })
}

fn prepare_font_artifact(
    scenario: &TextDocument,
    translated_scenario: &TextDocument,
    spd: &TextDocument,
    translated_spd: &TextDocument,
    spg: &TextDocument,
    translated_spg: &TextDocument,
    files: &[SnapshotFile],
) -> Result<(Option<EncodingPlan>, Option<FontArtifact>), String> {
    let baseline_texts = document_texts(scenario)
        .chain(document_texts(spd))
        .chain(document_texts(spg));
    let translated_texts = document_texts(translated_scenario)
        .chain(document_texts(translated_spd))
        .chain(document_texts(translated_spg));
    let translated_vec = translated_texts.collect::<Vec<_>>();
    let missing = characters_needing_carriers(translated_vec.iter().map(String::as_str));
    if missing.is_empty() {
        return Ok((None, None));
    }
    let baseline_vec = baseline_texts.collect::<Vec<_>>();
    let reserved = native_double_byte_slots(baseline_vec.iter().map(String::as_str));
    let substitutions = SubstitutionMap::embedded()?;
    let plan = EncodingPlan::build(
        &substitutions,
        reserved.iter().copied(),
        translated_vec.iter().map(String::as_str),
    )
    .map_err(|error| format!("PC-98 字库映射规划失败: {error}"))?;
    let font_source = files
        .iter()
        .find(|file| is_font_file(&file.relative))
        .map(|file| file.bytes.as_slice())
        .unwrap_or(EMBEDDED_FONT);
    let build = font_98::prepare_font(font_source, &plan.requests(), &reserved, FONT_FACE)?;
    let mapping = FontMappingDocument {
        _format: "platinum-star-pc98-font-mapping-v1".into(),
        tool_version: env!("CARGO_PKG_VERSION").into(),
        base_font_sha256: sha256(font_source),
        output_font_sha256: sha256(&build.bytes),
        entries: plan
            .manifest_entries()
            .map_err(|error| format!("字库映射清单生成失败: {error}"))?,
    };
    let relative = files
        .iter()
        .find(|file| is_font_file(&file.relative))
        .map(|file| file.relative.clone())
        .unwrap_or_else(|| PathBuf::from("font.bmp"));
    Ok((
        Some(plan),
        Some(FontArtifact {
            relative,
            bytes: build.bytes,
            mapping,
            patched_glyphs: build.patched_glyphs,
            reserved_cp932_slots: reserved.len(),
        }),
    ))
}

fn is_font_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.eq_ignore_ascii_case("font.bmp") || name.eq_ignore_ascii_case("font.tmp")
        })
}

fn document_texts<'a>(document: &'a TextDocument) -> impl Iterator<Item = String> + 'a {
    document.entries.iter().flat_map(|entry| {
        [
            entry.message.clone(),
            entry.name.clone().unwrap_or_default(),
        ]
        .into_iter()
    })
}

fn validate_translation_document(
    baseline: &TextDocument,
    translated: &TextDocument,
) -> Result<(), String> {
    if baseline.entries.len() != translated.entries.len() {
        return Err(format!(
            "{} 条目数量不一致：来源 {}，翻译 {}",
            baseline._file,
            baseline.entries.len(),
            translated.entries.len()
        ));
    }
    let mut normalized = translated.clone();
    for (index, (base, edit)) in baseline
        .entries
        .iter()
        .zip(&mut normalized.entries)
        .enumerate()
    {
        if base.name.is_some() != edit.name.is_some() {
            return Err(format!(
                "{} 记录 {} 不能新增或删除姓名字段",
                baseline._file, index
            ));
        }
        edit.name = base.name.clone();
        edit.message = base.message.clone();
    }
    if &normalized != baseline {
        return Err(format!(
            "{} 除 name/message 外的来源或定位字段被修改；请从当前来源重新导出后再编辑",
            baseline._file
        ));
    }
    Ok(())
}

fn read_document(path: &Path) -> Result<TextDocument, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("无法读取翻译文件 {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("翻译 JSON 无效 {}: {error}", path.display()))
}

fn validate_export_manifest(root: &Path) -> Result<(), String> {
    let path = root.join(TEXT_MANIFEST_FILENAME);
    let bytes = fs::read(&path)
        .map_err(|error| format!("翻译目录缺少有效文本清单 {}: {error}", path.display()))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("文本清单 JSON 无效 {}: {error}", path.display()))?;
    if value.get("_format").and_then(|value| value.as_str()) != Some(TEXT_MANIFEST_FORMAT) {
        return Err(format!("文本清单格式不匹配: {}", path.display()));
    }
    Ok(())
}

fn imported_file(
    file: &str,
    source: &[u8],
    output: &[u8],
    changed_entries: usize,
    output_form: &str,
) -> ImportedFile {
    ImportedFile {
        file: file.into(),
        source_sha256: sha256(source),
        output_sha256: sha256(output),
        changed_entries: changed_entries as u64,
        output_form: output_form.into(),
    }
}

fn exported_file(source: &str, output: &str, document: &TextDocument) -> ExportedFile {
    ExportedFile {
        source: source.into(),
        output: output.into(),
        source_sha256: document._source_sha256.clone(),
        decoded_sha256: document._decoded_sha256.clone(),
        entries: document.entries.len() as u64,
    }
}

fn read_source(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("无法读取来源文件 {}: {error}", path.display()))
}

fn snapshot_source_tree(root: &Path) -> Result<(Vec<PathBuf>, Vec<SnapshotFile>), String> {
    fn visit(
        root: &Path,
        directory: &Path,
        directories: &mut Vec<PathBuf>,
        files: &mut Vec<SnapshotFile>,
    ) -> Result<(), String> {
        let mut children = fs::read_dir(directory)
            .map_err(|error| format!("无法读取来源目录 {}: {error}", directory.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("无法枚举来源目录 {}: {error}", directory.display()))?;
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            let path = child.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("无法读取来源成员元数据 {}: {error}", path.display()))?;
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "来源目录包含符号链接，拒绝跟随: {}",
                    path.display()
                ));
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|_| format!("来源成员越过根目录: {}", path.display()))?
                .to_path_buf();
            if metadata.is_dir() {
                directories.push(relative);
                visit(root, &path, directories, files)?;
            } else if metadata.is_file() {
                if relative == Path::new(TEXT_IMPORT_MANIFEST_FILENAME) {
                    continue;
                }
                files.push(SnapshotFile {
                    relative,
                    bytes: fs::read(&path)
                        .map_err(|error| format!("无法读取来源文件 {}: {error}", path.display()))?,
                });
            } else {
                return Err(format!("来源目录包含不支持的成员类型: {}", path.display()));
            }
        }
        Ok(())
    }

    let mut directories = Vec::new();
    let mut files = Vec::new();
    visit(root, root, &mut directories, &mut files)?;
    Ok((directories, files))
}

fn snapshot_file<'a>(files: &'a [SnapshotFile], name: &str) -> Result<&'a [u8], String> {
    files
        .iter()
        .find(|file| file.relative == Path::new(name))
        .map(|file| file.bytes.as_slice())
        .ok_or_else(|| format!("游戏来源根目录缺少 {name}"))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("无法生成 JSON {}: {error}", path.display()))?;
    bytes.push(b'\n');
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("无法创建 {}: {error}", path.display()))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("无法写入 {}: {error}", path.display()))
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建目录 {}: {error}", parent.display()))?;
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("无法创建 {}: {error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("无法写入 {}: {error}", path.display()))
}

fn verify_json(path: &Path) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|error| format!("无法复核 {}: {error}", path.display()))?;
    serde_json::from_slice::<serde_json::Value>(&bytes)
        .map(|_| ())
        .map_err(|error| format!("写出后的 JSON 无效 {}: {error}", path.display()))
}

fn validate_managed_output(output: &Path) -> Result<(), String> {
    if !output.is_dir() {
        return Err(format!("已有输出不是目录: {}", output.display()));
    }
    let path = output.join(TEXT_MANIFEST_FILENAME);
    let bytes = fs::read(&path).map_err(|_| {
        format!(
            "拒绝覆盖非本工具管理的目录（缺少 {}）: {}",
            TEXT_MANIFEST_FILENAME,
            output.display()
        )
    })?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("已有文本清单无效 {}: {error}", path.display()))?;
    if value.get("_format").and_then(|value| value.as_str()) != Some(TEXT_MANIFEST_FORMAT) {
        return Err(format!(
            "拒绝覆盖清单格式不匹配的目录: {}",
            output.display()
        ));
    }
    Ok(())
}

fn validate_managed_import_output(output: &Path) -> Result<(), String> {
    if !output.is_dir() {
        return Err(format!("已有输出不是目录: {}", output.display()));
    }
    let path = output.join(TEXT_IMPORT_MANIFEST_FILENAME);
    let bytes = fs::read(&path).map_err(|_| {
        format!(
            "拒绝覆盖非本工具管理的目录（缺少 {}）: {}",
            TEXT_IMPORT_MANIFEST_FILENAME,
            output.display()
        )
    })?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("已有回注清单无效 {}: {error}", path.display()))?;
    if value.get("_format").and_then(|value| value.as_str()) != Some(TEXT_IMPORT_MANIFEST_FORMAT) {
        return Err(format!(
            "拒绝覆盖回注清单格式不匹配的目录: {}",
            output.display()
        ));
    }
    Ok(())
}

fn validate_sibling(output: &Path, candidate: &Path) -> Result<(), String> {
    if output.parent() != candidate.parent() || output == candidate {
        return Err("事务目录不在输出的同级位置".into());
    }
    let name = candidate
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "事务目录名不是有效 Unicode".to_string())?;
    if !name.contains(".platinum-star-text.") {
        return Err("事务目录名没有预期的安全标记".into());
    }
    Ok(())
}

fn safe_remove_dir(output: &Path, candidate: &Path) -> Result<(), String> {
    validate_sibling(output, candidate)?;
    if candidate.exists() {
        fs::remove_dir_all(candidate)
            .map_err(|error| format!("无法清理事务目录 {}: {error}", candidate.display()))?;
    }
    Ok(())
}

fn validate_import_sibling(output: &Path, candidate: &Path) -> Result<(), String> {
    if output.parent() != candidate.parent() || output == candidate {
        return Err("回注事务目录不在输出的同级位置".into());
    }
    let name = candidate
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "回注事务目录名不是有效 Unicode".to_string())?;
    if !name.contains(".platinum-star-import.") {
        return Err("回注事务目录名没有预期的安全标记".into());
    }
    Ok(())
}

fn safe_remove_import_dir(output: &Path, candidate: &Path) -> Result<(), String> {
    validate_import_sibling(output, candidate)?;
    if candidate.exists() {
        fs::remove_dir_all(candidate)
            .map_err(|error| format!("无法清理回注事务目录 {}: {error}", candidate.display()))?;
    }
    Ok(())
}

fn sha256(data: &[u8]) -> String {
    format!("{:X}", Sha256::digest(data))
}
