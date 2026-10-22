use crate::basic::{
    extract_translation, inject_translation, renderer_control_bytes, translation_index,
    untranslated_count, InjectionStats, TranslationFile, PATCHABLE_SCRIPT_NAMES, SCRIPT_NAMES,
};
use crate::font::{build_font_patches, FontPatchReport};
use crate::n88::{N88Disk, RebuildReport};
use crate::text_compression::{TextCompressionReport, TextDictionary};
use crate::{
    commit_directory, ensure_output_available, read_file, sha256_hex, staging_path, Result,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const EXPECTED_D88_SHA256: &str =
    "80F2BC41CE055F1238832FDA4FD0937173D9B1700ADD5E24A1B1A6D4C1CF1CE8";
pub const EXPECTED_FONT_SHA256: &str =
    "BAFEBB50034BA7763030E51B6CD4ACEEE9462317F8E1E164C85E720A13C0F8F8";

#[derive(Debug, Deserialize)]
struct UnpackManifest {
    format: String,
    source_sha256: String,
    files: Vec<UnpackFile>,
}

#[derive(Debug, Deserialize)]
struct UnpackFile {
    name: String,
    sha256: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct TranslationManifest {
    format: String,
    source_d88_sha256: String,
    scripts: Vec<TranslationManifestScript>,
}

#[derive(Debug, Serialize)]
struct TranslationManifestScript {
    script: String,
    json: String,
    source_sha256: String,
    entries: usize,
}

#[derive(Debug, Clone)]
pub struct PreparedTranslationExport {
    output: PathBuf,
    overwrite: bool,
    source_sha256: String,
    files: Vec<(String, TranslationFile)>,
}

#[derive(Debug, Clone)]
pub struct ExportReport {
    pub scripts: usize,
    pub entries: usize,
    pub output_root: PathBuf,
}

impl PreparedTranslationExport {
    pub fn scripts(&self) -> usize {
        self.files.len()
    }

    pub fn entries(&self) -> usize {
        self.files.iter().map(|(_, file)| file.entries.len()).sum()
    }

    pub fn write(self) -> Result<ExportReport> {
        ensure_output_available(&self.output, self.overwrite)?;
        let script_count = self.files.len();
        let entry_count = self.entries();
        let staging = staging_path(&self.output, "staging")?;
        fs::create_dir(&staging)
            .map_err(|error| format!("无法创建临时输出 {}: {error}", staging.display()))?;
        let result = (|| {
            let mut manifest_scripts = Vec::new();
            for (filename, file) in &self.files {
                let bytes = serde_json::to_vec_pretty(file)
                    .map_err(|error| format!("无法序列化 {filename}: {error}"))?;
                fs::write(staging.join(filename), [bytes.as_slice(), b"\n"].concat())
                    .map_err(|error| format!("无法写出 {filename}: {error}"))?;
                manifest_scripts.push(TranslationManifestScript {
                    script: file.script.clone(),
                    json: filename.clone(),
                    source_sha256: file.source_sha256.clone(),
                    entries: file.entries.len(),
                });
            }
            let manifest = TranslationManifest {
                format: "karuizawa-translation-workspace-v2".into(),
                source_d88_sha256: self.source_sha256.clone(),
                scripts: manifest_scripts,
            };
            write_json(&staging.join("translation_manifest.json"), &manifest)?;
            fs::write(staging.join("README.md"), translation_readme())
                .map_err(|error| format!("无法写出翻译说明：{error}"))?;
            commit_directory(&staging, &self.output, self.overwrite)
        })();
        if result.is_err() && staging.exists() {
            let _ = fs::remove_dir_all(&staging);
        }
        result?;
        Ok(ExportReport {
            scripts: script_count,
            entries: entry_count,
            output_root: self.output,
        })
    }
}

pub fn prepare_translation_export(
    workspace: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<PreparedTranslationExport> {
    ensure_output_available(output, overwrite)?;
    let manifest_path = workspace.join("manifest.json");
    let manifest: UnpackManifest = serde_json::from_slice(&read_file(&manifest_path)?)
        .map_err(|error| format!("无法解析 {}: {error}", manifest_path.display()))?;
    if manifest.format != "karuizawa-syndrome-n88-files-v1" {
        return Err(format!("不支持的解包工作区格式：{}", manifest.format));
    }
    if manifest.source_sha256 != EXPECTED_D88_SHA256 {
        return Err(format!(
            "解包工作区源盘哈希不属于已确认版本：{}",
            manifest.source_sha256
        ));
    }
    let by_name = manifest
        .files
        .iter()
        .map(|file| (file.name.to_ascii_uppercase(), file))
        .collect::<BTreeMap<_, _>>();
    let mut files = Vec::new();
    for (index, script) in SCRIPT_NAMES.iter().enumerate() {
        let record = by_name
            .get(&script.to_ascii_uppercase())
            .ok_or_else(|| format!("解包工作区缺少脚本 {script}"))?;
        let relative = safe_relative(&record.path)?;
        let path = workspace.join(relative);
        let data = read_file(&path)?;
        let actual_sha = sha256_hex(&data);
        if actual_sha != record.sha256 {
            return Err(format!(
                "{} 与 manifest 哈希不一致：清单={}，实际={actual_sha}",
                path.display(),
                record.sha256
            ));
        }
        let translation = extract_translation(script, &data)?;
        files.push((
            format!("{:02}_{}.json", index + 1, filename_script(script)),
            translation,
        ));
    }
    Ok(PreparedTranslationExport {
        output: output.to_path_buf(),
        overwrite,
        source_sha256: manifest.source_sha256,
        files,
    })
}

#[derive(Debug, Clone, Serialize)]
struct ScriptBuildReport {
    script: String,
    entries: usize,
    changed_entries: usize,
    untranslated_entries: usize,
    dimension_patches: usize,
    name_input_patches: usize,
    program_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
struct BuildJsonReport {
    format: String,
    source_d88_sha256: String,
    source_font_sha256: String,
    output_d88_sha256: String,
    scripts: Vec<ScriptBuildReport>,
    font: FontPatchReport,
    text_compression: TextCompressionReport,
    changed_files: usize,
    allocated_new_clusters: usize,
    remaining_free_clusters: usize,
    untranslated_entries: usize,
    structural_verification: String,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PreparedLocalizedBuild {
    output: PathBuf,
    overwrite: bool,
    source_name: String,
    source_sha256: String,
    font_sha256: String,
    rebuilt: RebuildReport,
    scripts: Vec<ScriptBuildReport>,
    font_report: FontPatchReport,
    text_compression_report: TextCompressionReport,
    replacement_names: Vec<String>,
    untranslated: usize,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BuildReport {
    pub scripts: usize,
    pub entries: usize,
    pub changed_entries: usize,
    pub changed_files: usize,
    pub output_root: PathBuf,
    pub output_d88: PathBuf,
    pub allocated_new_clusters: usize,
    pub warnings: Vec<String>,
}

impl PreparedLocalizedBuild {
    pub fn scripts(&self) -> usize {
        self.scripts.len()
    }

    pub fn entries(&self) -> usize {
        self.scripts.iter().map(|item| item.entries).sum()
    }

    pub fn changed_entries(&self) -> usize {
        self.scripts.iter().map(|item| item.changed_entries).sum()
    }

    pub fn untranslated_entries(&self) -> usize {
        self.untranslated
    }

    pub fn allocated_new_clusters(&self) -> usize {
        self.rebuilt.allocated_new_clusters
    }

    pub fn write(self) -> Result<BuildReport> {
        ensure_output_available(&self.output, self.overwrite)?;
        let script_count = self.scripts.len();
        let entry_count = self.entries();
        let changed_entry_count = self.changed_entries();
        let staging = staging_path(&self.output, "staging")?;
        fs::create_dir(&staging)
            .map_err(|error| format!("无法创建临时输出 {}: {error}", staging.display()))?;
        let output_filename = format!("{}_english.d88", self.source_name);
        let output_d88 = self.output.join(&output_filename);
        let result = (|| {
            fs::write(staging.join(&output_filename), &self.rebuilt.bytes)
                .map_err(|error| format!("无法写出成品 D88：{error}"))?;
            let rebuilt_disk = N88Disk::parse(self.rebuilt.bytes.clone())?;
            let patched = staging.join("patched_files");
            fs::create_dir(&patched).map_err(|error| format!("无法创建 patched_files：{error}"))?;
            for name in &self.replacement_names {
                let file = rebuilt_disk.file(name)?;
                fs::write(patched.join(filename_script(name)), &file.data)
                    .map_err(|error| format!("无法写出 {}：{error}", name))?;
            }
            let report = BuildJsonReport {
                format: "karuizawa-localized-build-v1".into(),
                source_d88_sha256: self.source_sha256.clone(),
                source_font_sha256: self.font_sha256.clone(),
                output_d88_sha256: sha256_hex(&self.rebuilt.bytes),
                scripts: self.scripts.clone(),
                font: self.font_report.clone(),
                text_compression: self.text_compression_report.clone(),
                changed_files: self.rebuilt.changed_files,
                allocated_new_clusters: self.rebuilt.allocated_new_clusters,
                remaining_free_clusters: self.rebuilt.remaining_free_clusters,
                untranslated_entries: self.untranslated,
                structural_verification:
                    "vn-d88 container decode + N88 directory/FAT/file read-back passed".into(),
                warnings: self.warnings.clone(),
            };
            write_json(&staging.join("report.json"), &report)?;
            fs::write(
                staging.join("README.md"),
                format!(
                    "# 軽井沢シンドローム English build\n\n挂载 `{output_filename}` 启动。`report.json` 记录源盘/字库哈希、脚本与 N88 回封结果。\n"
                ),
            )
            .map_err(|error| format!("无法写出成品说明：{error}"))?;
            commit_directory(&staging, &self.output, self.overwrite)
        })();
        if result.is_err() && staging.exists() {
            let _ = fs::remove_dir_all(&staging);
        }
        result?;
        Ok(BuildReport {
            scripts: script_count,
            entries: entry_count,
            changed_entries: changed_entry_count,
            changed_files: self.rebuilt.changed_files,
            output_root: self.output,
            output_d88,
            allocated_new_clusters: self.rebuilt.allocated_new_clusters,
            warnings: self.warnings,
        })
    }
}

#[derive(Default)]
pub struct BuildOptions {
    pub overwrite: bool,
    pub allow_incomplete: bool,
    pub no_compression: bool,
    pub large_font: bool,
}

pub fn prepare_localized_build(
    source: &Path,
    translations: &[PathBuf],
    font_rom: &Path,
    output: &Path,
    options: BuildOptions,
) -> Result<PreparedLocalizedBuild> {
    let BuildOptions {
        overwrite,
        allow_incomplete,
        no_compression,
        large_font,
    } = options;
    ensure_output_available(output, overwrite)?;
    let source_bytes = read_file(source)?;
    let source_sha256 = sha256_hex(&source_bytes);
    if source_sha256 != EXPECTED_D88_SHA256 {
        return Err(format!(
            "源 D88 不是已确认版本：{source_sha256}（预期 {EXPECTED_D88_SHA256}）"
        ));
    }
    let font_bytes = read_file(font_rom)?;
    let font_sha256 = sha256_hex(&font_bytes);
    if font_sha256 != EXPECTED_FONT_SHA256 {
        return Err(format!(
            "FONT.ROM 不是实验确认版本：{font_sha256}（预期 {EXPECTED_FONT_SHA256}）"
        ));
    }
    let disk = N88Disk::parse(source_bytes)?;
    let translation_files = load_translation_files(translations)?;
    let translations = translation_index(translation_files)?;
    for name in translations.keys() {
        if !SCRIPT_NAMES
            .iter()
            .any(|script| script.eq_ignore_ascii_case(name))
        {
            return Err(format!("翻译 JSON 指向未知脚本 {name}"));
        }
    }
    let mut reserved_renderer_bytes = HashSet::new();
    for script in PATCHABLE_SCRIPT_NAMES {
        reserved_renderer_bytes.extend(renderer_control_bytes(&disk.file(script)?.data)?);
    }
    let patchable_translations = translations
        .iter()
        .filter(|(name, _)| {
            PATCHABLE_SCRIPT_NAMES
                .iter()
                .any(|script| script.eq_ignore_ascii_case(name))
        })
        .map(|(name, file)| (name.clone(), file.clone()))
        .collect::<BTreeMap<_, _>>();
    let text_dictionary = if no_compression {
        TextDictionary::default()
    } else {
        TextDictionary::build(&patchable_translations, &reserved_renderer_bytes)
    };

    let mut replacements = HashMap::new();
    let mut additions = HashMap::new();
    let mut script_reports = Vec::new();
    let mut total_untranslated = 0usize;
    let mut injection_errors = Vec::new();
    for script in PATCHABLE_SCRIPT_NAMES {
        let source_file = disk.file(script)?;
        let baseline = extract_translation(script, &source_file.data)?;
        let selected = translations
            .get(&script.to_ascii_uppercase())
            .unwrap_or(&baseline);
        let untranslated = untranslated_count(selected);
        total_untranslated += untranslated;
        match inject_translation(
            script,
            &source_file.data,
            selected,
            &text_dictionary,
            large_font,
        ) {
            Ok((program, stats)) => {
                replacements.insert(script.to_string(), program);
                script_reports.push(script_report(script, stats, untranslated));
            }
            Err(error) => injection_errors.push(error),
        }
    }
    if !injection_errors.is_empty() {
        return Err(format!(
            "译文回注检查发现问题：\n{}",
            injection_errors
                .iter()
                .map(|error| format!("- {error}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    if total_untranslated != 0 && !allow_incomplete {
        return Err(format!(
            "仍有 {total_untranslated} 条含日文的 message 未修改；完成翻译后再生成正式盘，或仅为阶段测试启用 allow-incomplete"
        ));
    }

    let font_patches = build_font_patches(&disk, &font_bytes, &text_dictionary, large_font)?;
    let text_compression_report = text_dictionary.report(font_patches.decoder_address)?;
    replacements.insert("M2DATA".into(), font_patches.m2data);
    replacements.insert("M3DATA".into(), font_patches.m3data);
    if let Some(m3dict) = font_patches.m3dict {
        additions.insert("M3DICT".into(), m3dict);
    }
    let mut replacement_names = replacements.keys().cloned().collect::<Vec<_>>();
    replacement_names.extend(additions.keys().cloned());
    replacement_names.sort();
    let rebuilt = disk.rebuild_with_additions(&replacements, &additions)?;
    let source_name = source
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("源 D88 文件名无法表示为 Unicode：{}", source.display()))?
        .to_owned();
    let mut warnings = vec![
        "D 文件承载特殊保护轨数据，构建器强制逐字节保留，不作为普通 BASIC 脚本回注。".into(),
        if large_font {
            "本次为 16x12 大字诊断构建，正文重新换行并分页；F200h 字典仍需后续章节实机验证。".into()
        } else {
            "当前版本把 M2DATA 的 18 字节字形槽改写为 8px 英文字形；未翻译的日文消息不会正常显示。"
                .into()
        },
        "英文姓名在 KAL1 输入后会转为与正文相同的高位字形码，并连同定界符保存供后续章节读取。"
            .into(),
        if no_compression {
            "本次为无运行时字典的诊断构建；正文按未压缩高位字形码回注。".into()
        } else {
            format!(
                "正文使用 {} 项单字节字节对字典，构建时节省 {} 字节；启动盘载入预展开词条。",
                text_compression_report.dictionary_entries,
                text_compression_report.saved_text_bytes
            )
        },
    ];
    if total_untranslated != 0 {
        warnings.push(format!(
            "本次以 allow-incomplete 生成，仍有 {total_untranslated} 条日文 message。"
        ));
    }
    Ok(PreparedLocalizedBuild {
        output: output.to_path_buf(),
        overwrite,
        source_name,
        source_sha256,
        font_sha256,
        rebuilt,
        scripts: script_reports,
        font_report: font_patches.report,
        text_compression_report,
        replacement_names,
        untranslated: total_untranslated,
        warnings,
    })
}

fn script_report(script: &str, stats: InjectionStats, untranslated: usize) -> ScriptBuildReport {
    ScriptBuildReport {
        script: script.into(),
        entries: stats.entries,
        changed_entries: stats.changed_entries,
        untranslated_entries: untranslated,
        dimension_patches: stats.dimension_patches,
        name_input_patches: stats.name_input_patches,
        program_bytes: stats.program_bytes,
    }
}

fn load_translation_files(selected: &[PathBuf]) -> Result<Vec<TranslationFile>> {
    if selected.is_empty() {
        return Err("至少选择一个翻译 JSON 或目录".into());
    }
    let mut paths = Vec::new();
    for item in selected {
        let canonical = fs::canonicalize(item)
            .map_err(|error| format!("无法解析翻译输入 {}: {error}", item.display()))?;
        if canonical.is_file() {
            if is_translation_json(&canonical) {
                paths.push(canonical);
            } else {
                return Err(format!("翻译输入不是 JSON：{}", canonical.display()));
            }
        } else if canonical.is_dir() {
            let mut children = fs::read_dir(&canonical)
                .map_err(|error| format!("无法读取翻译目录 {}: {error}", canonical.display()))?
                .map(|entry| entry.map(|value| value.path()))
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|error| format!("无法枚举翻译目录 {}: {error}", canonical.display()))?;
            children.sort();
            paths.extend(
                children
                    .into_iter()
                    .filter(|path| is_translation_json(path)),
            );
        } else {
            return Err(format!("翻译输入不是文件或目录：{}", canonical.display()));
        }
    }
    let mut seen = HashSet::new();
    paths.retain(|path| seen.insert(path.clone()));
    if paths.is_empty() {
        return Err("选择范围内没有脚本翻译 JSON".into());
    }
    paths
        .into_iter()
        .map(|path| {
            serde_json::from_slice(&read_file(&path)?)
                .map_err(|error| format!("无法解析翻译 JSON {}: {error}", path.display()))
        })
        .collect()
}

fn is_translation_json(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("json"))
        && path
            .file_name()
            .and_then(|value| value.to_str())
            .is_none_or(|value| !value.eq_ignore_ascii_case("translation_manifest.json"))
}

fn safe_relative(value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("manifest 含有不安全相对路径：{value}"));
    }
    Ok(path.to_path_buf())
}

fn filename_script(script: &str) -> String {
    script
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("无法序列化 {}: {error}", path.display()))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|error| format!("无法写出 {}: {error}", path.display()))
}

fn translation_readme() -> &'static str {
    "# 軽井沢シンドローム translation workspace\n\n只编辑各脚本 JSON 条目的 `message`；`source`、`source_raw_hex`、定位和哈希均用于回注校验。\n\n`source`/`message` 已过滤空串、纯空格和纯控制串。颜色、换行、等待、缩进、说话人选择及斜杠状态不会进入译文，回注时由工具按原 BASIC 结构保留。原盘半角片假名已通过内置字典转成全角片假名；这只是方便阅读，不改变受保护的原始字节。\n\n把 `message` 改为半角 ASCII 文本即可，不要加入控制标签。工具把 `!`–`~` 映射到 M2DATA 的独立高位字槽；`print` 条目直接使用 PC-88 系统 FONT.ROM。\n"
}
