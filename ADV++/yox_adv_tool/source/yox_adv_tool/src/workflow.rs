//! YOX export/import orchestration used by both CLI and the interactive panel.

use crate::{
    carrier::{cp932_decode, cp932_encode, plan_carriers_with_codes, slot_for_code},
    font::YoxFont,
    renderer,
    script::Script,
    Archive,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use vn_cli::*;

fn output_path(requested: &Path) -> Result<PathBuf> {
    let requested = std::path::absolute(requested)?;
    let parent = fs::canonicalize(requested.parent().ok_or("输出缺少父目录")?)?;
    Ok(parent.join(requested.file_name().ok_or("输出缺少文件名")?))
}

fn source_name(path: &Path) -> Result<String> {
    Ok(path
        .file_name()
        .ok_or("来源缺少文件名")?
        .to_string_lossy()
        .into_owned())
}

fn sidecars(template: &Path) -> Result<(PathBuf, PathBuf)> {
    let parent = template.parent().ok_or("模板路径缺少父目录")?;
    Ok((parent.join("primary.json"), parent.join("source.dat")))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
struct PrimaryRecord {
    index: usize,
    fid: usize,
    string_slot: usize,
    original_message: String,
    #[serde(default)]
    projected: bool,
    #[serde(default)]
    hidden_suffix: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
struct PrimaryManifest {
    #[serde(default)]
    projection_contract: u32,
    #[serde(default)]
    projection_stats: crate::script::ProjectionStats,
    source_file: String,
    source_size: usize,
    script_count: usize,
    records: Vec<PrimaryRecord>,
}

fn parse_script_archive(path: &Path) -> Result<(Archive, Vec<Script>)> {
    let archive = Archive::parse(source_name(path)?, fs::read(path)?)?;
    if archive.entries.is_empty() || archive.entries.iter().any(|e| e.kind != "yox_script") {
        return Err("来源档案必须只包含有效的 YOX 脚本条目".into());
    }
    let scripts = (0..archive.entries.len())
        .map(|id| Script::parse(archive.entry_bytes(id)?).map_err(Error::from))
        .collect::<Result<Vec<_>>>()?;
    Ok((archive, scripts))
}

fn parse_font_archive(path: &Path) -> Result<(Archive, Vec<YoxFont>)> {
    let archive = Archive::parse(source_name(path)?, fs::read(path)?)?;
    if archive.entries.len() != 6 || archive.entries.iter().any(|e| e.kind != "yox_font") {
        return Err("字体档案必须包含6份有效YOX字库".into());
    }
    let dimensions = [(15, 16), (20, 22), (24, 26), (27, 29), (53, 57), (40, 43)];
    let fonts = (0..6)
        .map(|id| {
            let font = YoxFont::parse(archive.entry_bytes(id)?.to_vec()).map_err(Error::from)?;
            if (font.width, font.height) != dimensions[id] {
                return Err(format!("字库 {id} 尺寸异常").into());
            }
            Ok(font)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((archive, fonts))
}

pub struct Export;

impl Operation for Export {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "export",
            "导出消息翻译模板",
            vec![
                Field::new("source", "原始 script.dat", FieldKind::Path).required(),
                Field::new("output", "新的导出工作目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "按消息指令提取3F文本；不导出资源名或图形姓名，输出符合 vn-text 的纯翻译数组。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("source", Value::Path(path.clone()));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err("导出模板不支持覆盖，请选择新文件".into());
        }
        let source = fs::canonicalize(parameters.path("source")?)?;
        let output = output_path(parameters.path("output")?)?;
        if output.try_exists()? || output.starts_with(&source) || source.starts_with(&output) {
            return Err("输出必须是新的独立目录".into());
        }
        let (archive, scripts) = parse_script_archive(&source)?;
        let projections = scripts
            .iter()
            .map(Script::translation_projection)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::from)?;
        let projection_stats = projections.iter().fold(
            crate::script::ProjectionStats::default(),
            |mut total, current| {
                total.total_messages += current.stats.total_messages;
                total.exported_messages += current.stats.exported_messages;
                total.filtered_layout += current.stats.filtered_layout;
                total.filtered_blank += current.stats.filtered_blank;
                total.filtered_control_only += current.stats.filtered_control_only;
                total
            },
        );
        let entries: Vec<_> = projections
            .iter()
            .flat_map(|projection| {
                projection.messages.iter().map(|message| vn_text::Entry {
                    name: None,
                    message: message.message.clone(),
                })
            })
            .collect();
        if entries.is_empty() {
            return Err("脚本档案没有可导出的普通正文".into());
        }
        let bytes = vn_text::write_template(&entries).map_err(|e| Error(e.to_string()))?;
        let mut records = Vec::with_capacity(entries.len());
        for (fid, (script, projection)) in scripts.iter().zip(&projections).enumerate() {
            let selected: BTreeMap<usize, _> = projection
                .messages
                .iter()
                .map(|message| (message.reference_index, message))
                .collect();
            for (reference_index, slot) in script.message_refs.iter().copied().enumerate() {
                let message = selected.get(&reference_index);
                records.push(PrimaryRecord {
                    index: records.len(),
                    fid,
                    string_slot: slot,
                    original_message: script.strings[slot].text.clone(),
                    projected: message.is_some(),
                    hidden_suffix: message
                        .map(|record| record.suffix.clone())
                        .unwrap_or_default(),
                });
            }
        }
        let total_records = records.len();
        let manifest = PrimaryManifest {
            projection_contract: 2,
            projection_stats: projection_stats.clone(),
            source_file: source_name(&source)?,
            source_size: archive.raw_bytes().len(),
            script_count: scripts.len(),
            records,
        };
        let manifest_bytes =
            serde_json::to_vec_pretty(&manifest).map_err(|e| Error(e.to_string()))?;
        let snapshot = archive.raw_bytes().to_vec();
        let preview = Preview {
            inputs: vec![source],
            outputs: vec![output.clone()],
            steps: vec![
                "解析所有脚本和3F引用".into(),
                "保存原生主提取sidecar".into(),
                "写出vn-text数组".into(),
            ],
            details: vec![format!(
                "{}个脚本，全部{}条3F引用中导出{}条正文、过滤布局{}/空白{}/纯控制{}；sidecar保存全量原始FID/槽和来源DAT，不计算哈希",
                scripts.len(),
                total_records,
                entries.len(),
                projections.iter().map(|p| p.stats.filtered_layout).sum::<usize>(),
                projections.iter().map(|p| p.stats.filtered_blank).sum::<usize>(),
                projections.iter().map(|p| p.stats.filtered_control_only).sum::<usize>(),
            )],
        };
        Ok(Box::new(ExportJob {
            preview,
            output,
            bytes,
            manifest_bytes,
            snapshot,
            count: entries.len(),
            projection_stats,
        }))
    }
}

struct ExportJob {
    preview: Preview,
    output: PathBuf,
    bytes: Vec<u8>,
    manifest_bytes: Vec<u8>,
    snapshot: Vec<u8>,
    count: usize,
    projection_stats: crate::script::ProjectionStats,
}

impl PreparedOperation for ExportJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }
    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("写入消息翻译模板及主提取sidecar")?;
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let parent = self.output.parent().ok_or("输出目录缺少父目录")?;
        let stage = loop {
            let path = parent.join(format!(
                ".yox-export-{}-{}.tmp",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => break StageDir(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        };
        fs::write(stage.0.join("translations.json"), &self.bytes)?;
        fs::write(stage.0.join("primary.json"), &self.manifest_bytes)?;
        fs::write(stage.0.join("source.dat"), &self.snapshot)?;
        if self.output.try_exists()? {
            return Err("输出目录在执行期间出现；拒绝覆盖".into());
        }
        fs::rename(&stage.0, &self.output)?;
        Ok(RunReport {
            summary: "消息模板导出完成。".into(),
            totals: vec![
                ("投影正文".into(), self.count as u64),
                (
                    "3F引用总数".into(),
                    self.projection_stats.total_messages as u64,
                ),
                (
                    "过滤布局消息".into(),
                    self.projection_stats.filtered_layout as u64,
                ),
                (
                    "过滤空白消息".into(),
                    self.projection_stats.filtered_blank as u64,
                ),
                (
                    "过滤纯控制消息".into(),
                    self.projection_stats.filtered_control_only as u64,
                ),
            ],
            outputs: vec![self.output.clone()],
            ..RunReport::default()
        })
    }
}

#[derive(Serialize)]
struct CarrierRecord {
    source: String,
    cp932: String,
    carrier: String,
    glyph_slot: usize,
}

#[derive(Serialize)]
struct CarrierPlanSummary {
    mask: String,
    valid_addressable_slots: usize,
    reserved_existing_slots: usize,
    allocated_carrier_slots: usize,
    remaining_free_slots: usize,
}
#[derive(Serialize)]
struct CarrierPlanFile {
    summary: CarrierPlanSummary,
    mappings: Vec<CarrierRecord>,
}

pub struct Import;
impl Operation for Import {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "import",
            "注回译文并重绘字体",
            vec![
                Field::new("script-source", "原始 script.dat", FieldKind::Path).required(),
                Field::new("font-source", "原始 font.dat", FieldKind::Path).required(),
                Field::new("translation", "译文 JSON", FieldKind::Path).required(),
                Field::new("wrap-width", "消息窗口宽度（像素）", FieldKind::Text)
                    .required()
                    .default(Value::Text("1053".into())),
                Field::new(
                    "font-slot",
                    "用于排版测宽的字库编号",
                    FieldKind::Choice((0..6).map(|n| n.to_string()).collect()),
                )
                .required()
                .default(Value::Text("3".into())),
                Field::new("output", "新的重建输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "保留全库所有未翻译字符槽，按一套CP932计划重建脚本与6份新宋体字库；窗口像素宽度和活动字号需明确指定。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            parameters.set("script-source", Value::Path(path.clone()));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err("导入只写入新目录，不支持覆盖".into());
        }
        let script_path = fs::canonicalize(parameters.path("script-source")?)?;
        let font_path = fs::canonicalize(parameters.path("font-source")?)?;
        let translation_path = fs::canonicalize(parameters.path("translation")?)?;
        let (manifest_requested, snapshot_requested) = sidecars(&translation_path)?;
        let manifest_path = fs::canonicalize(manifest_requested)?;
        let snapshot_path = fs::canonicalize(snapshot_requested)?;
        let output = output_path(parameters.path("output")?)?;
        if output.try_exists()?
            || [&script_path, &font_path, &translation_path]
                .iter()
                .any(|p| output.starts_with(p) || p.starts_with(&output))
        {
            return Err("输出必须是新的且与所有输入分离的目录".into());
        }
        let (script_archive, scripts) = parse_script_archive(&script_path)?;
        if scripts.len() != 1263 {
            return Err(format!("脚本数为{}，预期1263", scripts.len()).into());
        }
        let manifest: PrimaryManifest = serde_json::from_slice(&fs::read(&manifest_path)?)
            .map_err(|e| Error(format!("主提取sidecar无效：{e}")))?;
        if manifest.projection_contract != 2 {
            return Err(format!(
                "主提取投影契约版本{}不支持；请从当前来源重新export",
                manifest.projection_contract
            )
            .into());
        }
        let snapshot = fs::read(&snapshot_path)?;
        if snapshot != script_archive.raw_bytes() {
            return Err(
                "script.dat与导出模板绑定的原始二进制快照不同；拒绝按同形状译文注回".into(),
            );
        }
        let mut source_records = Vec::new();
        for (fid, script) in scripts.iter().enumerate() {
            let projection = script.translation_projection().map_err(Error::from)?;
            let selected: BTreeMap<usize, _> = projection
                .messages
                .iter()
                .map(|message| (message.reference_index, message))
                .collect();
            for (reference_index, slot) in script.message_refs.iter().copied().enumerate() {
                let message = selected.get(&reference_index);
                source_records.push(PrimaryRecord {
                    index: source_records.len(),
                    fid,
                    string_slot: slot,
                    original_message: script.strings[slot].text.clone(),
                    projected: message.is_some(),
                    hidden_suffix: message
                        .map(|record| record.suffix.clone())
                        .unwrap_or_default(),
                });
            }
        }
        if manifest.projection_contract != 2
            || manifest.source_file != source_name(&script_path)?
            || manifest.source_size != snapshot.len()
            || manifest.script_count != scripts.len()
            || manifest.records != source_records
        {
            return Err(
                "主提取sidecar的来源、投影契约、FID/槽顺序或原文记录与当前脚本不一致".into(),
            );
        }
        let (font_archive, fonts) = parse_font_archive(&font_path)?;
        let projections = scripts
            .iter()
            .map(Script::translation_projection)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::from)?;
        let expected: Vec<_> = projections
            .iter()
            .flat_map(|projection| {
                projection.messages.iter().map(|message| vn_text::Entry {
                    name: None,
                    message: message.message.clone(),
                })
            })
            .collect();
        let supplied = vn_text::read_template(&fs::read(&translation_path)?)
            .map_err(|e| Error(e.to_string()))?;
        vn_text::validate_shape(&expected, &supplied).map_err(|e| Error(e.to_string()))?;
        for (index, entry) in supplied.iter().enumerate() {
            if entry.name.is_some() {
                return Err(format!("译文第{index}项包含不可用姓名字段").into());
            }
            if entry.message.contains('\0') {
                return Err(format!("译文第{index}项含NUL").into());
            }
        }
        let mut final_text_scripts = scripts.clone();
        let mut translated_at = 0usize;
        let mut changed_slots = Vec::<BTreeSet<usize>>::with_capacity(scripts.len());
        for (script, projection) in final_text_scripts.iter_mut().zip(&projections) {
            let start = translated_at;
            let end = start
                .checked_add(projection.messages.len())
                .ok_or("消息索引溢出")?;
            let entries = supplied
                .get(start..end)
                .ok_or("翻译记录数与脚本索引不一致")?;
            changed_slots.push(
                script
                    .apply_projected_messages(entries)
                    .map_err(Error::from)?
                    .into_iter()
                    .collect(),
            );
            translated_at = end;
        }
        if translated_at != supplied.len() {
            return Err("未消费全部译文记录".into());
        }
        let preserved = final_text_scripts
            .iter()
            .flat_map(Script::preserved_text)
            .collect::<Vec<_>>();
        let raw_codes = scripts
            .iter()
            .zip(&changed_slots)
            .map(|(script, changed)| {
                script
                    .strings
                    .iter()
                    .enumerate()
                    .filter(|(slot, _)| !changed.contains(slot))
                    .try_fold(Vec::new(), |mut all, (_, slot)| {
                        all.extend(crate::carrier::codes_in_text(&slot.bytes)?);
                        Ok::<Vec<u16>, String>(all)
                    })
                    .map_err(Error::from)
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let translated = supplied
            .iter()
            .map(|entry| entry.message.as_str())
            .collect::<Vec<_>>();
        let mut reserved_slots = BTreeSet::new();
        let mut valid_slots = 0usize;
        for slot in 0..9589 {
            let code = crate::carrier::code_for_slot(slot, 0x3d).expect("validated game mask");
            if cp932_decode(code).is_some() {
                valid_slots += 1;
            }
        }
        for text in &preserved {
            for ch in text.chars() {
                if let Some(code) = cp932_encode(ch) {
                    if crate::carrier::slot_for_code(code, 0x3d).is_some() {
                        reserved_slots.insert(crate::carrier::slot_for_code(code, 0x3d).unwrap());
                    }
                }
            }
        }
        for code in &raw_codes {
            if let Some(slot) = crate::carrier::slot_for_code(*code, 0x3d) {
                reserved_slots.insert(slot);
            }
        }
        for text in &translated {
            for ch in text.chars() {
                if let Some(code) = cp932_encode(ch) {
                    if let Some(slot) = crate::carrier::slot_for_code(code, 0x3d) {
                        reserved_slots.insert(slot);
                    }
                }
            }
        }
        let carriers = plan_carriers_with_codes(0x3d, preserved, raw_codes, translated)
            .map_err(Error::from)?;
        let allocated_slots = carriers
            .values()
            .filter_map(|code| crate::carrier::slot_for_code(*code, 0x3d))
            .collect::<BTreeSet<_>>();
        if allocated_slots.len() != carriers.len()
            || allocated_slots.iter().any(|s| reserved_slots.contains(s))
        {
            return Err("载体映射与保留槽冲突".into());
        }
        let plan_summary = CarrierPlanSummary {
            mask: "0x3D".into(),
            valid_addressable_slots: valid_slots,
            reserved_existing_slots: reserved_slots.len(),
            allocated_carrier_slots: allocated_slots.len(),
            remaining_free_slots: valid_slots
                .saturating_sub(reserved_slots.len() + allocated_slots.len()),
        };
        let width_pixels: usize = parameters
            .text("wrap-width")?
            .parse()
            .map_err(|_| Error("--wrap-width 必须是正整数像素".into()))?;
        if width_pixels == 0 {
            return Err("--wrap-width 必须大于0".into());
        }
        let font_slot: usize = parameters
            .text("font-slot")?
            .parse()
            .map_err(|_| Error("--font-slot 必须是0..5".into()))?;
        if font_slot >= 6 {
            return Err("--font-slot 必须是0..5".into());
        }
        let preview = Preview {
            inputs: vec![
                script_path,
                font_path,
                translation_path,
                manifest_path,
                snapshot_path,
            ],
            outputs: vec![output.clone()],
            steps: vec![
                "预检完整脚本及译文形状".into(),
                "规划全库动态CP932载体".into(),
                "按新宋体重绘6份字库并更新宽度".into(),
                "按选定像素宽度重新生成@L".into(),
                "重建script.dat/font.dat并原子提交".into(),
            ],
            details: vec![format!(
                "1263个脚本，{}条消息记录，{}个载体字符，字体{}，窗口{}px",
                expected.len(),
                carriers.len(),
                font_slot,
                width_pixels
            )],
        };
        Ok(Box::new(ImportJob {
            preview,
            output,
            script_archive,
            scripts,
            font_archive,
            fonts,
            supplied,
            carriers,
            plan_summary,
            width_pixels,
            font_slot,
        }))
    }
}

struct ImportJob {
    preview: Preview,
    output: PathBuf,
    script_archive: Archive,
    scripts: Vec<Script>,
    font_archive: Archive,
    fonts: Vec<YoxFont>,
    supplied: Vec<vn_text::Entry>,
    carriers: std::collections::BTreeMap<char, u16>,
    plan_summary: CarrierPlanSummary,
    width_pixels: usize,
    font_slot: usize,
}

struct StageDir(PathBuf);
impl Drop for StageDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl PreparedOperation for ImportJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }
    fn execute(mut self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("使用新宋体重绘六份字库")?;
        let mut font_replacements = Vec::new();
        let mut redrawn_metrics = Vec::new();
        let mut preview_files = Vec::new();
        let mut fallback_records = Vec::new();
        let sample_chars: Vec<char> = "国中文gjpq，。、.,（）「」\"0123ABC".chars().collect();
        for (fid, font) in self.fonts.iter().enumerate() {
            progress.report(&format!("重绘字库 {fid}（{}×{}）", font.width, font.height))?;
            let redrawn = renderer::redraw(font, &self.carriers).map_err(Error::from)?;
            for record in &redrawn.fallbacks {
                let mut value = serde_json::to_value(record).map_err(|e| Error(e.to_string()))?;
                value["font_id"] = serde_json::json!(fid);
                value["font_size"] = serde_json::json!([font.width, font.height]);
                fallback_records.push(value);
            }
            let rebuilt = font
                .rebuild(&redrawn.glyphs, &redrawn.advances)
                .map_err(Error::from)?;
            let measured = YoxFont::parse(rebuilt.clone()).map_err(Error::from)?;
            font_replacements.push((fid, rebuilt));
            let mut sample_pixels = Vec::new();
            let mut sample_metrics = Vec::new();
            for ch in &sample_chars {
                let code = self
                    .carriers
                    .get(ch)
                    .copied()
                    .or_else(|| cp932_encode(*ch))
                    .ok_or_else(|| Error(format!("预览字符 {ch} 无CP932载体")))?;
                let slot = slot_for_code(code, font.range_mask)
                    .ok_or_else(|| Error(format!("预览字符 {ch} 不在字库范围")))?;
                let pixels = redrawn
                    .glyphs
                    .get(slot)
                    .ok_or_else(|| Error(format!("预览字形槽 {slot} 缺失")))?
                    .1
                    .clone();
                let advance = redrawn
                    .advances
                    .get(slot)
                    .ok_or_else(|| Error(format!("预览字宽槽 {slot} 缺失")))?
                    .1;
                sample_pixels.push(pixels);
                sample_metrics.push(serde_json::json!({"char": ch.to_string(), "slot": slot, "advance_px": advance}));
            }
            let line_chars: Vec<char> = "中文，下一句。ABC,def.gjpq".chars().collect();
            let mut line_pixels = Vec::new();
            let mut line_advances = Vec::new();
            for ch in &line_chars {
                let code = self
                    .carriers
                    .get(ch)
                    .copied()
                    .or_else(|| cp932_encode(*ch))
                    .ok_or_else(|| Error(format!("连排预览字符 {ch} 无CP932载体")))?;
                let slot = slot_for_code(code, font.range_mask)
                    .ok_or_else(|| Error(format!("连排预览字符 {ch} 不在字库范围")))?;
                line_pixels.push(redrawn.glyphs[slot].1.clone());
                line_advances.push(redrawn.advances[slot].1);
            }
            preview_files.push((
                fid,
                font.width,
                font.height,
                redrawn.face,
                sample_pixels,
                sample_metrics,
                line_pixels,
                line_advances,
            ));
            redrawn_metrics.push(measured);
        }
        progress.report("按重绘后的字宽安全插入@L")?;
        let mut translated_at = 0usize;
        for script in &mut self.scripts {
            let count = script
                .translation_projection()
                .map_err(Error::from)?
                .messages
                .len();
            let end = translated_at.checked_add(count).ok_or("消息索引溢出")?;
            let entries = self
                .supplied
                .get(translated_at..end)
                .ok_or("翻译记录数与脚本索引不一致")?;
            script
                .apply_projected_messages_reflow(
                    entries,
                    self.width_pixels,
                    &redrawn_metrics[self.font_slot],
                    &self.carriers,
                )
                .map_err(Error::from)?;
            translated_at = end;
        }
        if translated_at != self.supplied.len() {
            return Err("未消费全部译文记录".into());
        }
        let mut script_replacements = Vec::with_capacity(self.scripts.len());
        for (fid, script) in self.scripts.iter().enumerate() {
            script_replacements.push((fid, script.rebuild(&self.carriers).map_err(Error::from)?));
        }
        let script_dat = self.script_archive.rebuild(&script_replacements)?;
        let font_dat = self.font_archive.rebuild(&font_replacements)?;

        static NEXT: AtomicU64 = AtomicU64::new(0);
        let parent = self
            .output
            .parent()
            .ok_or("输出目录缺少父目录")?
            .to_path_buf();
        let stage = loop {
            let path = parent.join(format!(
                ".yox-import-{}-{}.tmp",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => break StageDir(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        };
        fs::write(stage.0.join("script.dat"), script_dat)?;
        fs::write(stage.0.join("font.dat"), font_dat)?;
        let mapping = self
            .carriers
            .iter()
            .map(|(source, code)| {
                let carrier = cp932_decode(*code)
                    .ok_or_else(|| Error(format!("CP932载体 {code:04X} 无效")))?;
                let slot = slot_for_code(*code, 0x3d)
                    .ok_or_else(|| Error(format!("载体 {code:04X} 不在游戏字库范围")))?;
                Ok(CarrierRecord {
                    source: source.to_string(),
                    cp932: format!("{code:04X}"),
                    carrier: carrier.to_string(),
                    glyph_slot: slot,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let plan = CarrierPlanFile {
            summary: self.plan_summary,
            mappings: mapping,
        };
        fs::write(
            stage.0.join("carrier_map.json"),
            serde_json::to_vec_pretty(&plan).map_err(|e| Error(e.to_string()))?,
        )?;
        fs::write(
            stage.0.join("font_fallbacks.json"),
            serde_json::to_vec_pretty(&fallback_records).map_err(|e| Error(e.to_string()))?,
        )?;
        let previews = stage.0.join("previews");
        fs::create_dir(&previews)?;
        for (fid, width, height, face, pixels, metrics, line_pixels, line_advances) in preview_files
        {
            let image_path = previews.join(format!("font-{fid:02}-{width}x{height}.bmp"));
            fs::write(
                &image_path,
                renderer::samples_bmp(width, height, &pixels).map_err(Error::from)?,
            )?;
            let line_path = previews.join(format!("font-{fid:02}-{width}x{height}-line.bmp"));
            fs::write(
                &line_path,
                renderer::line_preview_bmp(width, height, &line_pixels, &line_advances)
                    .map_err(Error::from)?,
            )?;
            let metadata = serde_json::json!({"font_face": face, "font_id": fid, "cell": [width,height], "baseline": "shared GDI ascent/descent baseline; punctuation ink anchored to bottom-right of its advance", "advance_source": "measured per glyph and stored as new font table advance", "characters": metrics, "line_text": "中文，下一句。ABC,def.gjpq", "line_advances_px": line_advances});
            fs::write(
                image_path.with_extension("json"),
                serde_json::to_vec_pretty(&metadata).map_err(|e| Error(e.to_string()))?,
            )?;
        }
        if self.output.try_exists()? {
            return Err("输出目录在执行期间出现；拒绝覆盖".into());
        }
        fs::rename(&stage.0, &self.output)?;
        Ok(RunReport {
            summary: "脚本注回和六字号字库重绘完成。".into(),
            totals: vec![
                ("脚本数".into(), self.scripts.len() as u64),
                ("消息记录".into(), self.supplied.len() as u64),
                ("载体字符".into(), self.carriers.len() as u64),
                ("重绘字库".into(), 6),
            ],
            outputs: vec![self.output.clone()],
            ..RunReport::default()
        })
    }
}
