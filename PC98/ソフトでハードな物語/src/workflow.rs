//! Translation workspace export, VM reassembly, PC-98 font generation and FDI rebuild.
use crate::{text_vm, translation};
use pc98_fdi_unpack::{extract_fdi_files, rebuild_fdi};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs,
    fs::OpenOptions,
    io::Write,
    path::{Component, Path, PathBuf},
};
use vn_font::font_98::{self, EncodingPlan, SubstitutionMap};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug)]
struct ProtectedInput {
    path: PathBuf,
    hash: String,
}

#[derive(Debug)]
struct DiskSnapshot {
    id: String,
    path: PathBuf,
    source_name: String,
    bytes: Vec<u8>,
    hash: String,
    files: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug)]
struct ScriptWork {
    disk_id: String,
    path: String,
    bytes: Vec<u8>,
    hash: String,
    decoded: text_vm::DecodeDocument,
    edits: Vec<translation::StringEdit>,
}

#[derive(Debug)]
pub struct Prepared {
    target: PathBuf,
    protected: Vec<ProtectedInput>,
    files: BTreeMap<String, Vec<u8>>,
    summary: String,
    totals: Vec<(String, u64)>,
}

impl Prepared {
    pub fn inputs(&self) -> Vec<PathBuf> {
        self.protected
            .iter()
            .map(|item| item.path.clone())
            .collect()
    }

    pub fn output(&self) -> &Path {
        &self.target
    }

    pub fn execute(self, progress: &mut dyn vn_cli::Progress) -> vn_cli::Result<vn_cli::RunReport> {
        progress.report("重新核对原盘、译文和工作区快照")?;
        for input in &self.protected {
            let actual = hash_path(&input.path).map_err(vn_cli::Error)?;
            if actual != input.hash {
                return Err(format!("操作期间输入发生变化: {}", input.path.display()).into());
            }
        }
        progress.report("写入暂存目录并提交镜像、字库和报告")?;
        commit_directory(&self.target, &self.files).map_err(vn_cli::Error)?;
        Ok(vn_cli::RunReport {
            summary: self.summary,
            totals: self.totals,
            outputs: vec![self.target],
            ..vn_cli::RunReport::default()
        })
    }
}

pub fn prepare_export(disk_a: &Path, disk_b: &Path, output: &Path) -> Result<Prepared> {
    let disks = vec![read_disk("SOFT_A", disk_a)?, read_disk("SOFT_B", disk_b)?];
    let output = prepare_target(output, disks.iter().map(|disk| disk.path.as_path()))?;
    let protected = disks
        .iter()
        .map(|disk| ProtectedInput {
            path: disk.path.clone(),
            hash: disk.hash.clone(),
        })
        .collect::<Vec<_>>();
    let mut files = BTreeMap::new();
    let mut baselines = Vec::new();
    let mut script_count = 0usize;
    let mut text_count = 0usize;
    for disk in &disks {
        for (path, bytes) in &disk.files {
            if !is_script_path(path) {
                continue;
            }
            let digest = sha256(bytes);
            let mut decoded = text_vm::parse(bytes, format!("{}/{path}", disk.id));
            let template = translation::make_template_from_decoded(
                &disk.id,
                path,
                bytes,
                digest.clone(),
                &mut decoded,
            );
            let template_path = template_path(&disk.id, path)?;
            let string_count = decoded
                .instructions
                .iter()
                .map(|instruction| instruction.strings.len())
                .sum::<usize>();
            let encoded = json_bytes(&template)?;
            files.insert(template_path.clone(), encoded);
            baselines.push(translation::ScriptBaseline {
                disk_id: disk.id.clone(),
                path: path.clone(),
                byte_length: bytes.len() as u64,
                sha256: digest,
                template_path,
                instruction_count: decoded.instructions.len(),
                string_count,
            });
            script_count += 1;
            text_count += string_count;
        }
    }
    if baselines.is_empty() {
        return Err("两张镜像里没有找到已确认的 TXT/CHR/ITEM/HELP/SEND 脚本".into());
    }
    baselines.sort_by(|left, right| {
        left.disk_id
            .cmp(&right.disk_id)
            .then_with(|| left.path.cmp(&right.path))
    });
    let manifest = translation::WorkspaceManifest {
        schema: translation::WORKSPACE_SCHEMA.into(),
        tool_version: env!("CARGO_PKG_VERSION").into(),
        disks: disks
            .iter()
            .map(|disk| translation::DiskBaseline {
                id: disk.id.clone(),
                source_name: disk.source_name.clone(),
                byte_length: disk.bytes.len() as u64,
                sha256: disk.hash.clone(),
            })
            .collect(),
        scripts: baselines,
    };
    files.insert("workspace.json".into(), json_bytes(&manifest)?);
    files.insert("README.md".into(), export_readme().as_bytes().to_vec());
    for relative in files.keys() {
        validate_relative_output(relative)?;
    }
    Ok(Prepared {
        target: output,
        protected,
        files,
        summary: "译文工作区已生成".into(),
        totals: vec![
            ("脚本文件".into(), script_count as u64),
            ("可编辑文本串".into(), text_count as u64),
        ],
    })
}

pub fn prepare_build(
    disk_a: &Path,
    disk_b: &Path,
    translation_root: &Path,
    output: &Path,
    face: &str,
) -> Result<Prepared> {
    if face.trim().is_empty() {
        return Err("字体名称不能为空".into());
    }
    let translation_root = canonical_directory(translation_root)?;
    let workspace_path = translation_root.join("workspace.json");
    let workspace_bytes = read_regular_file(&workspace_path)?;
    let workspace: translation::WorkspaceManifest = serde_json::from_slice(&workspace_bytes)
        .map_err(|e| format!("{}: {e}", workspace_path.display()))?;
    if workspace.schema != translation::WORKSPACE_SCHEMA {
        return Err(format!("不支持的翻译工作区格式: {}", workspace.schema));
    }
    if workspace.disks.len() != 2 || workspace.scripts.is_empty() {
        return Err("workspace.json 缺少两张源盘或脚本基线".into());
    }
    let disk_a = read_disk("SOFT_A", disk_a)?;
    let disk_b = read_disk("SOFT_B", disk_b)?;
    let disks = vec![disk_a, disk_b];
    let output = prepare_target(
        output,
        disks
            .iter()
            .map(|disk| disk.path.as_path())
            .chain(std::iter::once(translation_root.as_path())),
    )?;
    let expected_disks = workspace
        .disks
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect::<HashMap<_, _>>();
    if expected_disks.len() != 2
        || !expected_disks.contains_key("SOFT_A")
        || !expected_disks.contains_key("SOFT_B")
    {
        return Err("workspace.json 的盘标识必须恰好为 SOFT_A 和 SOFT_B".into());
    }
    for disk in &disks {
        let baseline = expected_disks[disk.id.as_str()];
        if disk.hash != baseline.sha256 || disk.bytes.len() as u64 != baseline.byte_length {
            return Err(format!(
                "{} 与 workspace.json 中的原盘 SHA-256/大小不匹配",
                disk.id
            ));
        }
    }

    let mut baseline_keys = HashSet::new();
    let mut expected_template_paths = BTreeSet::new();
    let mut script_work = Vec::new();
    for baseline in &workspace.scripts {
        if !matches!(baseline.disk_id.as_str(), "SOFT_A" | "SOFT_B")
            || !is_script_path(&baseline.path)
            || !baseline_keys.insert((baseline.disk_id.clone(), baseline.path.clone()))
        {
            return Err("workspace.json 含有重复或不支持的脚本路径".into());
        }
        validate_template_path(&baseline.template_path)?;
        if !expected_template_paths.insert(baseline.template_path.clone()) {
            return Err("workspace.json 的翻译模板路径重复".into());
        }
        let disk = disks
            .iter()
            .find(|disk| disk.id == baseline.disk_id)
            .ok_or("workspace.json 引用了未知盘")?;
        let bytes = disk
            .files
            .get(&baseline.path)
            .ok_or_else(|| format!("原盘缺少脚本 {}:{}", baseline.disk_id, baseline.path))?
            .clone();
        let digest = sha256(&bytes);
        if digest != baseline.sha256 || bytes.len() as u64 != baseline.byte_length {
            return Err(format!(
                "脚本基线不匹配: {}:{}",
                baseline.disk_id, baseline.path
            ));
        }
        let mut decoded = text_vm::parse(&bytes, format!("{}/{}", baseline.disk_id, baseline.path));
        let template = translation::make_template_from_decoded(
            &baseline.disk_id,
            &baseline.path,
            &bytes,
            digest.clone(),
            &mut decoded,
        );
        if decoded.instructions.len() != baseline.instruction_count
            || decoded
                .instructions
                .iter()
                .map(|instruction| instruction.strings.len())
                .sum::<usize>()
                != baseline.string_count
        {
            return Err(format!("脚本解码结构与工作区清单不匹配: {}", baseline.path));
        }
        let template_path = translation_root.join(&baseline.template_path);
        let supplied = read_regular_file(&template_path)?;
        let edits = translation::edits_from_template(&supplied, &template, &decoded)
            .map_err(|e| format!("{}: {e}", template_path.display()))?;
        script_work.push(ScriptWork {
            disk_id: baseline.disk_id.clone(),
            path: baseline.path.clone(),
            bytes,
            hash: digest,
            decoded,
            edits,
        });
    }

    let current_script_keys = disks
        .iter()
        .flat_map(|disk| {
            disk.files
                .keys()
                .filter(|path| is_script_path(path))
                .map(|path| (disk.id.clone(), path.clone()))
                .collect::<Vec<_>>()
        })
        .collect::<HashSet<_>>();
    if current_script_keys != baseline_keys {
        return Err("原盘中的脚本集合与 workspace.json 不一致".into());
    }
    let translation_dir = translation_root.join("translations");
    let actual_template_paths = walk_regular_files(&translation_dir)?
        .into_iter()
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        })
        .map(|path| {
            path.strip_prefix(&translation_root)
                .map(|relative| relative.to_string_lossy().replace('\\', "/"))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<BTreeSet<_>>>()?;
    if actual_template_paths != expected_template_paths {
        return Err("translations/ 中的 JSON 文件与 workspace.json 模板清单不一致".into());
    }

    let mut reserved = BTreeSet::new();
    let mut plan_texts = Vec::new();
    let mut requested_by_script =
        BTreeMap::<(String, String), BTreeMap<(usize, usize), String>>::new();
    let mut changed_entries = 0usize;
    for script in &script_work {
        for instruction in &script.decoded.instructions {
            reserved.extend(translation::collect_reserved_cp932(
                std::slice::from_ref(instruction),
                &script.bytes,
            ));
        }
        let edits = requested_by_script
            .entry((script.disk_id.clone(), script.path.clone()))
            .or_default();
        for edit in &script.edits {
            let instruction = script
                .decoded
                .instructions
                .iter()
                .find(|instruction| instruction.offset == edit.instruction_offset)
                .ok_or("翻译 JSON 指向不存在的指令")?;
            let payload = instruction
                .strings
                .iter()
                .find(|payload| payload.index.unwrap_or(0) == edit.string_index)
                .ok_or("翻译 JSON 指向不存在的文本串")?;
            let requested = translation::canonical_translation(&edit.text)?;
            if requested != payload.translation {
                plan_texts.push(translation::plan_text(&edit.text)?);
                changed_entries += 1;
            }
            if edits
                .insert(
                    (edit.instruction_offset, edit.string_index),
                    edit.text.clone(),
                )
                .is_some()
            {
                return Err("翻译 JSON 中同一条文本出现多次".into());
            }
        }
    }
    let plan = EncodingPlan::build_with_forbidden_cp932(
        &SubstitutionMap::embedded().map_err(|e| format!("内置 PC-98 字形映射: {e}"))?,
        reserved.iter().copied(),
        [],
        plan_texts.iter().map(String::as_str),
    )
    .map_err(|e| format!("无法规划翻译字槽: {e}"))?;
    let font = font_98::prepare_font(font_98::EMBEDDED_FONT, &plan.requests(), &reserved, face)
        .map_err(|e| format!("生成 NP2 PC-98 字库失败: {e}"))?;

    let mut replacements_a = BTreeMap::new();
    let mut replacements_b = BTreeMap::new();
    let mut changed_script_count = 0usize;
    let mut script_review = Vec::new();
    for script in &script_work {
        let key = (script.disk_id.clone(), script.path.clone());
        let supplied = requested_by_script.get(&key).ok_or("脚本缺少翻译记录")?;
        let (rebuilt, entry_changes) =
            reassemble_script(&script.bytes, &script.decoded, supplied, &plan)
                .map_err(|e| format!("{}:{}: {e}", script.disk_id, script.path))?;
        if let Some(rebuilt) = rebuilt {
            if rebuilt != script.bytes {
                changed_script_count += 1;
                let target = if script.disk_id == "SOFT_A" {
                    &mut replacements_a
                } else {
                    &mut replacements_b
                };
                target.insert(script.path.clone(), rebuilt.clone());
            }
        }
        script_review.push(json!({
            "disk_id": script.disk_id,
            "path": script.path,
            "source_sha256": script.hash,
            "changed_entries": entry_changes,
        }));
    }
    let rebuilt_a = rebuild_fdi(&disks[0].bytes, &disks[0].source_name, &replacements_a)
        .map_err(|e| format!("SOFT_A FDI 回封失败: {e}"))?;
    let rebuilt_b = rebuild_fdi(&disks[1].bytes, &disks[1].source_name, &replacements_b)
        .map_err(|e| format!("SOFT_B FDI 回封失败: {e}"))?;

    let mut output_files = BTreeMap::<String, Vec<u8>>::new();
    output_files.insert("SOFT_A_translated.FDI".into(), rebuilt_a.bytes.clone());
    output_files.insert("SOFT_B_translated.FDI".into(), rebuilt_b.bytes.clone());
    output_files.insert("font.tmp".into(), font.bytes.clone());
    output_files.insert(
        "font_mapping.json".into(),
        json_bytes(&plan.manifest_entries().map_err(|e| e.to_string())?)?,
    );
    let reserved_codes = reserved.iter().copied().collect::<Vec<_>>();
    let build_report = json!({
        "schema": "soft-hard-pc98-build-report-v1",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "source_disks": workspace.disks,
        "output_images": [
            {"path": "SOFT_A_translated.FDI", "sha256": sha256(&rebuilt_a.bytes), "changed_files": rebuilt_a.changed_files, "allocated_clusters": rebuilt_a.allocated_clusters, "released_clusters": rebuilt_a.released_clusters, "byte_identical_to_source": rebuilt_a.bytes == disks[0].bytes},
            {"path": "SOFT_B_translated.FDI", "sha256": sha256(&rebuilt_b.bytes), "changed_files": rebuilt_b.changed_files, "allocated_clusters": rebuilt_b.allocated_clusters, "released_clusters": rebuilt_b.released_clusters, "byte_identical_to_source": rebuilt_b.bytes == disks[1].bytes}
        ],
        "translated_scripts": changed_script_count,
        "translated_strings": changed_entries,
        "font": {
            "path": "font.tmp",
            "source_sha256": font_98::embedded_font_sha256(),
            "output_sha256": sha256(&font.bytes),
            "patched_glyphs": font.patched_glyphs,
            "face": face,
            "reserved_cp932": reserved_codes,
        },
        "scripts": script_review,
        "validation": "重解析全部修改脚本，核对非分支指令字段、分支目标、译文回读、FAT12 回封后的文件内容。"
    });
    output_files.insert("build_report.json".into(), json_bytes(&build_report)?);
    output_files.insert("README.md".into(), build_readme().as_bytes().to_vec());
    let mut protected = disks
        .iter()
        .map(|disk| ProtectedInput {
            path: disk.path.clone(),
            hash: disk.hash.clone(),
        })
        .collect::<Vec<_>>();
    protected.push(ProtectedInput {
        path: translation_root.clone(),
        hash: hash_path(&translation_root)?,
    });
    for relative in output_files.keys() {
        validate_relative_output(relative)?;
    }
    Ok(Prepared {
        target: output,
        protected,
        files: output_files,
        summary: "译文回注、PC-98 字库和两张 FDI 已准备完成".into(),
        totals: vec![
            ("修改脚本".into(), changed_script_count as u64),
            ("修改文本串".into(), changed_entries as u64),
            ("重绘字形".into(), font.patched_glyphs as u64),
            (
                "分配磁盘簇".into(),
                (rebuilt_a.allocated_clusters + rebuilt_b.allocated_clusters) as u64,
            ),
        ],
    })
}

fn reassemble_script(
    source: &[u8],
    document: &text_vm::DecodeDocument,
    edits: &BTreeMap<(usize, usize), String>,
    plan: &EncodingPlan,
) -> Result<(Option<Vec<u8>>, usize)> {
    let mut replacements = BTreeMap::<(usize, usize), Vec<u8>>::new();
    let mut expected_texts = BTreeMap::<(usize, usize), String>::new();
    let mut changed_entries = 0usize;
    for instruction in &document.instructions {
        for payload in &instruction.strings {
            let key = (instruction.offset, payload.index.unwrap_or(0));
            let requested = edits
                .get(&key)
                .ok_or_else(|| format!("缺少文本项 {:#x}/{}", key.0, key.1))?;
            let canonical = translation::canonical_translation(requested)?;
            if canonical == payload.translation {
                continue;
            }
            let encoded = translation::encode_translation(requested, plan)?;
            if translation::decoded_translation(&encoded, plan)? != canonical {
                return Err(format!("文本 {:#x}/{} 编码回读不一致", key.0, key.1));
            }
            replacements.insert(key, encoded);
            expected_texts.insert(key, canonical);
            changed_entries += 1;
        }
    }
    if edits.len()
        != document
            .instructions
            .iter()
            .map(|instruction| instruction.strings.len())
            .sum::<usize>()
    {
        return Err("翻译 JSON 中的文本项总数与当前脚本不一致".into());
    }
    if replacements.is_empty() {
        return Ok((None, 0));
    }
    if !matches!(&document.stop_reason, text_vm::StopReason::Terminator) {
        return Err("流未以 0x00 完整结束，拒绝变长回注".into());
    }
    if document
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error")
    {
        return Err("脚本含截断指令".into());
    }
    for instruction in &document.instructions {
        for branch in &instruction.branches {
            if !branch.target_is_instruction_boundary {
                return Err(format!(
                    "分支目标 {:#x} 不在已解析指令边界，无法安全移动指令",
                    branch.target_offset
                ));
            }
        }
    }

    let mut offset_map = BTreeMap::new();
    let mut new_starts = Vec::with_capacity(document.instructions.len());
    let mut new_lengths = Vec::with_capacity(document.instructions.len());
    let mut next_offset = 0usize;
    for instruction in &document.instructions {
        let mut length = instruction.end_offset - instruction.offset;
        for payload in &instruction.strings {
            let key = (instruction.offset, payload.index.unwrap_or(0));
            if let Some(encoded) = replacements.get(&key) {
                let old_length = payload.end_offset - payload.offset;
                length = length
                    .checked_sub(old_length)
                    .and_then(|value| value.checked_add(encoded.len()))
                    .ok_or("新脚本长度计算溢出")?;
            }
        }
        offset_map.insert(instruction.offset, next_offset);
        new_starts.push(next_offset);
        new_lengths.push(length);
        next_offset = next_offset.checked_add(length).ok_or("新脚本过大")?;
    }
    let original_stream_end = document
        .instructions
        .last()
        .ok_or("脚本没有指令")?
        .end_offset;
    let new_stream_end = next_offset;
    let mut rebuilt =
        Vec::with_capacity(new_stream_end + source.len().saturating_sub(original_stream_end));
    for (instruction_index, instruction) in document.instructions.iter().enumerate() {
        if rebuilt.len() != new_starts[instruction_index] {
            return Err("内部错误：指令偏移计划不连续".into());
        }
        let raw = &source[instruction.offset..instruction.end_offset];
        let mut local_replacements = instruction
            .strings
            .iter()
            .filter_map(|payload| {
                replacements
                    .get(&(instruction.offset, payload.index.unwrap_or(0)))
                    .map(|encoded| (payload.offset, payload.end_offset, encoded.as_slice()))
            })
            .collect::<Vec<_>>();
        local_replacements.sort_by_key(|replacement| replacement.0);
        let mut cursor = 0usize;
        let mut serialized = Vec::with_capacity(new_lengths[instruction_index]);
        for (start, end, encoded) in &local_replacements {
            let start = start - instruction.offset;
            let end = end - instruction.offset;
            if start < cursor || end < start || end > raw.len() {
                return Err("脚本内文本范围重叠或越界".into());
            }
            serialized.extend_from_slice(&raw[cursor..start]);
            serialized.extend_from_slice(encoded);
            cursor = end;
        }
        serialized.extend_from_slice(&raw[cursor..]);
        for branch in &instruction.branches {
            let target = *offset_map
                .get(&branch.target_offset)
                .ok_or_else(|| format!("分支目标 {:#x} 无法映射", branch.target_offset))?;
            let shift_before = local_replacements
                .iter()
                .filter(|(_, end, _)| *end <= branch.base_offset)
                .try_fold(0isize, |sum, (start, end, encoded)| {
                    let old = end.saturating_sub(*start);
                    let delta = encoded.len() as isize - old as isize;
                    sum.checked_add(delta).ok_or("分支位置偏移溢出")
                })?;
            let new_base = add_signed(
                new_starts[instruction_index]
                    .checked_add(branch.base_offset - instruction.offset)
                    .ok_or("分支基准偏移溢出")?,
                shift_before,
            )?;
            let signed_displacement = if target >= new_base {
                i64::try_from(target - new_base).map_err(|_| "分支位移超出计算范围")?
            } else {
                -i64::try_from(new_base - target).map_err(|_| "分支位移超出计算范围")?
            };
            let displacement = i16::try_from(signed_displacement).map_err(|_| {
                format!(
                    "分支 {:#x} 的有符号相对位移超出 16 位范围",
                    branch.operand_offset
                )
            })?;
            let operand_shift = local_replacements
                .iter()
                .filter(|(_, end, _)| *end <= branch.operand_offset)
                .try_fold(0isize, |sum, (start, end, encoded)| {
                    sum.checked_add(encoded.len() as isize - end.saturating_sub(*start) as isize)
                        .ok_or("分支操作数偏移溢出")
                })?;
            let relative_operand = branch.operand_offset - instruction.offset;
            let relative_operand = add_signed(relative_operand, operand_shift)?;
            let operand_end = relative_operand.checked_add(2).ok_or("分支操作数越界")?;
            if operand_end > serialized.len() {
                return Err("分支操作数超出重组后的指令".into());
            }
            serialized[relative_operand..operand_end].copy_from_slice(&displacement.to_be_bytes());
        }
        if serialized.len() != new_lengths[instruction_index] {
            return Err("内部错误：重组后的指令长度不匹配".into());
        }
        rebuilt.extend_from_slice(&serialized);
    }
    rebuilt.extend_from_slice(&source[original_stream_end..]);

    let output_document = text_vm::parse(&rebuilt, document.source.clone());
    if output_document.instructions.len() != document.instructions.len()
        || !matches!(
            &output_document.stop_reason,
            text_vm::StopReason::Terminator
        )
    {
        return Err("回注后重新解析的指令数量或终止位置改变".into());
    }
    for (old, new) in document
        .instructions
        .iter()
        .zip(&output_document.instructions)
    {
        if old.opcode != new.opcode || old.mnemonic != new.mnemonic {
            return Err("回注后命令流结构改变".into());
        }
        let old_operands = old
            .operands
            .iter()
            .filter(|operand| operand.name != "relative_displacement")
            .map(|operand| (&operand.name, operand.value))
            .collect::<Vec<_>>();
        let new_operands = new
            .operands
            .iter()
            .filter(|operand| operand.name != "relative_displacement")
            .map(|operand| (&operand.name, operand.value))
            .collect::<Vec<_>>();
        if old_operands != new_operands {
            return Err(format!("指令 {:#x} 的非分支参数发生变化", old.offset));
        }
        for branch in &old.branches {
            let mapped_target = *offset_map
                .get(&branch.target_offset)
                .ok_or("分支目标映射丢失")?;
            if !new.branches.iter().any(|rebuilt_branch| {
                rebuilt_branch.choice_index == branch.choice_index
                    && rebuilt_branch.target_offset == mapped_target
            }) {
                return Err(format!("指令 {:#x} 的分支目标回读不一致", old.offset));
            }
        }
    }
    for (old_instruction, new_instruction) in document
        .instructions
        .iter()
        .zip(&output_document.instructions)
    {
        for (position, (old_payload, new_payload)) in old_instruction
            .strings
            .iter()
            .zip(&new_instruction.strings)
            .enumerate()
        {
            let key = (old_instruction.offset, old_payload.index.unwrap_or(0));
            let Some(expected) = expected_texts.get(&key) else {
                continue;
            };
            let terminator = new_payload
                .terminator_offset
                .ok_or("回注文本缺少 NUL-u16 终止符")?;
            let text =
                translation::decoded_translation(&rebuilt[new_payload.offset..terminator], plan)?;
            if text != *expected {
                return Err(format!(
                    "文本 {:#x}/{} 回读不一致",
                    old_instruction.offset, position
                ));
            }
        }
    }
    Ok((Some(rebuilt), changed_entries))
}

fn add_signed(value: usize, delta: isize) -> Result<usize> {
    if delta >= 0 {
        value
            .checked_add(delta as usize)
            .ok_or_else(|| "偏移计算溢出".into())
    } else {
        value
            .checked_sub(delta.unsigned_abs())
            .ok_or_else(|| "偏移计算下溢".into())
    }
}

fn read_disk(id: &str, path: &Path) -> Result<DiskSnapshot> {
    let path = canonical_regular_file(path)?;
    let bytes = fs::read(&path).map_err(|e| format!("无法读取 {}: {e}", path.display()))?;
    let source_name = path
        .file_name()
        .ok_or("FDI 路径缺少文件名")?
        .to_string_lossy()
        .into_owned();
    let extracted =
        extract_fdi_files(&bytes, &source_name).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut files = BTreeMap::new();
    let mut casefolded = HashSet::new();
    for file in extracted {
        if !casefolded.insert(file.path.to_ascii_uppercase()) {
            return Err(format!("{} 中有重复成员路径 {}", path.display(), file.path));
        }
        files.insert(file.path, file.data);
    }
    Ok(DiskSnapshot {
        id: id.into(),
        path,
        source_name,
        hash: sha256(&bytes),
        bytes,
        files,
    })
}

fn is_script_path(path: &str) -> bool {
    if path.contains('/') || path.contains('\\') || !path.is_ascii() {
        return false;
    }
    let upper = path.to_ascii_uppercase();
    if !upper.ends_with(".DAT") {
        return false;
    }
    let stem = &upper[..upper.len() - 4];
    if matches!(stem, "HELP" | "SEND") {
        return true;
    }
    (stem.starts_with("TXT") && stem.len() == 8 && stem[3..].bytes().all(|b| b.is_ascii_digit()))
        || (stem.starts_with("CHR")
            && stem.len() == 6
            && stem[3..].bytes().all(|b| b.is_ascii_digit()))
        || (stem.starts_with("ITEM")
            && stem.len() == 7
            && stem[4..].bytes().all(|b| b.is_ascii_digit()))
}

fn template_path(disk_id: &str, path: &str) -> Result<String> {
    if !matches!(disk_id, "SOFT_A" | "SOFT_B") || !is_script_path(path) {
        return Err(format!("不支持的脚本路径: {disk_id}/{path}"));
    }
    Ok(format!("translations/{disk_id}/{path}.json"))
}

fn validate_template_path(path: &str) -> Result<()> {
    let normalized = Path::new(path);
    if !path.starts_with("translations/")
        || normalized
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || normalized
            .extension()
            .is_none_or(|extension| !extension.eq_ignore_ascii_case("json"))
    {
        return Err(format!("不安全的模板路径: {path}"));
    }
    Ok(())
}

fn json_bytes(value: &impl serde::Serialize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_regular_file(path: &Path) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    reject_link_metadata(path, &metadata)?;
    if !metadata.is_file() {
        return Err(format!("不是普通文件: {}", path.display()));
    }
    fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn canonical_directory(path: &Path) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    reject_link_metadata(path, &metadata)?;
    if !metadata.is_dir() {
        return Err(format!("不是目录: {}", path.display()));
    }
    fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn reject_link_metadata(path: &Path, metadata: &fs::Metadata) -> Result<()> {
    if metadata.file_type().is_symlink() {
        return Err(format!("不接受符号链接输入: {}", path.display()));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(format!("不接受重解析点输入: {}", path.display()));
        }
    }
    Ok(())
}

fn read_regular_file(path: &Path) -> Result<Vec<u8>> {
    canonical_regular_file(path)?;
    fs::read(path).map_err(|e| format!("无法读取 {}: {e}", path.display()))
}

fn walk_regular_files(root: &Path) -> Result<Vec<PathBuf>> {
    let root = canonical_directory(root)?;
    let mut result = Vec::new();
    walk_into(&root, &mut result)?;
    result.sort();
    Ok(result)
}

fn walk_into(path: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))?;
    for entry in entries {
        let child = entry.map_err(|e| e.to_string())?.path();
        let metadata = fs::symlink_metadata(&child).map_err(|e| e.to_string())?;
        reject_link_metadata(&child, &metadata)?;
        if metadata.is_dir() {
            walk_into(&child, result)?;
        } else if metadata.is_file() {
            result.push(child);
        } else {
            return Err(format!("不支持的输入文件类型: {}", child.display()));
        }
    }
    Ok(())
}

fn hash_path(path: &Path) -> Result<String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    reject_link_metadata(path, &metadata)?;
    if metadata.is_file() {
        return Ok(sha256(&fs::read(path).map_err(|e| e.to_string())?));
    }
    if !metadata.is_dir() {
        return Err(format!("输入路径不是普通文件或目录: {}", path.display()));
    }
    let canonical = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let files = walk_regular_files(&canonical)?;
    let mut hasher = Sha256::new();
    for file in files {
        let relative = file
            .strip_prefix(&canonical)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        hasher.update(Sha256::digest(fs::read(&file).map_err(|e| e.to_string())?));
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn prepare_target<'a>(output: &Path, inputs: impl Iterator<Item = &'a Path>) -> Result<PathBuf> {
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = canonical_directory(parent)?;
    let name = output.file_name().ok_or("输出路径必须指定新目录名")?;
    let target = parent.join(name);
    match fs::symlink_metadata(&target) {
        Ok(_) => return Err(format!("输出路径已存在: {}", target.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("{}: {error}", target.display())),
    }
    for input in inputs {
        let input = fs::canonicalize(input).map_err(|e| format!("{}: {e}", input.display()))?;
        if target.starts_with(&input) {
            return Err("输出目录不能位于源镜像或译文工作区内部".into());
        }
    }
    Ok(target)
}

fn validate_relative_output(path: &str) -> Result<()> {
    let path = Path::new(path);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("不安全的生成文件路径: {}", path.display()));
    }
    Ok(())
}

fn commit_directory(target: &Path, files: &BTreeMap<String, Vec<u8>>) -> Result<()> {
    if target.exists() {
        return Err(format!("输出目录已存在: {}", target.display()));
    }
    let parent = target.parent().ok_or("输出目录缺少父目录")?;
    let name = target
        .file_name()
        .ok_or("输出目录缺少名称")?
        .to_string_lossy();
    let mut staging = None;
    for suffix in 0..1000u32 {
        let candidate = parent.join(format!(
            ".{name}.soft-hard-pc98-{}-{suffix}.staging",
            std::process::id()
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => {
                staging = Some(candidate);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("无法创建暂存目录: {error}")),
        }
    }
    let staging = staging.ok_or("无法取得唯一暂存目录名")?;
    let write_result = (|| {
        for (relative, bytes) in files {
            validate_relative_output(relative)?;
            let path = staging.join(relative);
            let directory = path.parent().ok_or("输出文件缺少父目录")?;
            fs::create_dir_all(directory).map_err(|e| e.to_string())?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|e| format!("{}: {e}", path.display()))?;
            file.write_all(bytes).map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
        }
        match fs::symlink_metadata(target) {
            Ok(_) => return Err(format!("输出目录在提交前已被创建: {}", target.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{}: {error}", target.display())),
        }
        fs::rename(&staging, target).map_err(|e| format!("提交输出目录失败: {e}"))
    })();
    if write_result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    write_result
}

fn export_readme() -> &'static str {
    "# ソフトでハードな物語：译文工作区\n\n使用 UTF-8 编辑 `translations/SOFT_A/*.json` 和 `translations/SOFT_B/*.json` 中 `entries[]` 的 `message` 字段。每条只保留 `_index`、不可修改的原文 `scr_msg` 和可编辑译文 `message`；文件名、盘号、源哈希与编码标记用于校验，不能修改。指令、偏移、原始字节和分支结构由工具从原盘重新解析，不放进译文文件。\n\n脚本文本默认按 CP932 导出。可读字符（包括日文标点）直接显示；游戏自定义换行 `0x8197` 在 JSON 中表示为 `\\n`，回注时还原为原字码。无法按 CP932 解码的字码以 `[[PC98:XXXX]]` 保留。\n\n构建时仍需提供与 `workspace.json` SHA-256 匹配的两张原始 FDI；工具会输出两张译后 FDI 和与本次字槽映射配套的 `font.tmp`。原盘不会被覆盖。\n"
}

fn build_readme() -> &'static str {
    "# ソフトでハードな物語：构建产物\n\n将 `SOFT_A_translated.FDI` 和 `SOFT_B_translated.FDI` 分别挂载到 PC-98 模拟器软驱。请同时将本目录的 `font.tmp` 作为本次构建使用的 NP2 字库；它与 `font_mapping.json` 中的字槽映射配套。\n\n`build_report.json` 记录源盘、回封结果和结构检查摘要。镜像、字库与报告在一个新目录中提交，源盘和译文工作区保持不变。\n"
}
