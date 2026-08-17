use crate::bundle::{ApiEntry, Bundle, BundleError, CStrEntry, SourceInfo, TalkInfoRecord};
use crate::encoding::encode_cp932;
use crate::json_model::{
    editable_name, PuaInfo, TextReference, TranslationEntry, TranslationFile, TRANSLATION_FORMAT,
};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ExtractOptions {
    pub input: PathBuf,
    pub output: PathBuf,
    pub overwrite: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractReport {
    pub scanned_files: usize,
    pub json_files: usize,
    pub extracted_entries: usize,
    pub dialogue_entries: usize,
    pub choice_entries: usize,
    pub name_entries: usize,
    pub warnings: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ExtractPlan {
    pub translation_files: Vec<PlannedTranslationFile>,
    pub warnings: Vec<String>,
    pub report: ExtractReport,
}

#[derive(Debug, Clone)]
pub struct PlannedTranslationFile {
    pub relative_path: String,
    pub file_id: Option<usize>,
    pub file: Option<String>,
    pub group: String,
    pub translation: TranslationFile,
}

#[derive(Debug, Clone)]
struct Occurrence {
    group: String,
    file_id: usize,
    file: String,
    cstr_id: usize,
    type_name: String,
    speaker_name: Option<String>,
    name_cstr_id: Option<usize>,
    talk_info_id: Option<u32>,
    talk_style: Option<[u32; 2]>,
    message_id: Option<u32>,
    body_origin: Option<String>,
    call_offset: usize,
    source: SourceInfo,
    choice_group: Option<usize>,
    choice_slot: Option<usize>,
}

type OccurrenceKey = (
    String,
    usize,
    usize,
    String,
    Option<usize>,
    Option<[u32; 2]>,
);
type OccurrenceGroups = HashMap<OccurrenceKey, Vec<Occurrence>>;
type SourceGroups = BTreeMap<(String, usize), Vec<(Occurrence, Vec<Occurrence>)>>;

fn is_system_file(file: &str) -> bool {
    file.eq_ignore_ascii_case("_UserFunc.ss")
}

fn hex_offset(offset: usize) -> String {
    format!("0x{:08x}", offset)
}

fn pua_metadata(text: &str) -> Vec<PuaInfo> {
    let mut result = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for character in text.chars() {
        let codepoint = character as u32;
        if !(0xe000..=0xf8ff).contains(&codepoint) || !seen.insert(codepoint) {
            continue;
        }
        let character_text = character.to_string();
        let encoded = encode_cp932(&character_text, "PUA metadata").unwrap_or_default();
        let cp932_hex = encoded
            .iter()
            .map(|byte| format!("{:02X}", byte))
            .collect::<Vec<_>>()
            .join(" ");
        result.push(PuaInfo {
            character: character.to_string(),
            codepoint,
            cp932_hex,
        });
    }
    result
}

fn cstr_entry<'a>(
    bundle: &'a Bundle,
    id: usize,
    context: &str,
) -> Result<&'a CStrEntry, BundleError> {
    bundle
        .cstr
        .get(id)
        .ok_or_else(|| format!("{} 引用越界 CSTR[{}]", context, id))
}

fn make_reference(occurrence: &Occurrence) -> TextReference {
    TextReference {
        file: occurrence.file.clone(),
        file_id: occurrence.file_id,
        inst_offset: hex_offset(occurrence.call_offset),
        line: occurrence.source.line,
        dbg_record: occurrence.source.dbg_record,
        type_name: occurrence.type_name.clone(),
        choice_group: occurrence.choice_group,
        choice_slot: occurrence.choice_slot,
    }
}

fn build_entry(
    bundle: &Bundle,
    index: usize,
    first: &Occurrence,
    refs: &[Occurrence],
) -> Result<TranslationEntry, BundleError> {
    let cstr = cstr_entry(bundle, first.cstr_id, "文本条目")?;
    let name = if let Some(name_id) = first.name_cstr_id {
        Some(editable_name(
            &cstr_entry(bundle, name_id, "TalkInfo 名称")?.text,
        ))
    } else {
        first.speaker_name.clone()
    };
    let scr_name = name.clone();
    let type_name = first.type_name.clone();
    Ok(TranslationEntry {
        file: first.file.clone(),
        file_id: first.file_id,
        index,
        type_name,
        scr_name,
        name,
        message: cstr.text.clone(),
        scr_msg: cstr.text.clone(),
        cstr_id: first.cstr_id,
        name_cstr_id: first.name_cstr_id,
        talk_info_id: first.talk_info_id,
        talk_style: first.talk_style,
        message_id: first.message_id,
        offset: cstr.pool_offset,
        size: cstr.size.saturating_sub(1),
        cstr_size: cstr.size,
        inst_offset: hex_offset(first.call_offset),
        line: first.source.line,
        dbg_record: first.source.dbg_record,
        opcode: if first.message_id.is_some() {
            "B2/Mess$is".to_string()
        } else if first.type_name == "choice" || first.type_name == "system_choice" {
            first
                .body_origin
                .clone()
                .unwrap_or_else(|| "C6/sysSelect$iissssss".to_string())
        } else {
            "B2/dialogue_wrapper".to_string()
        },
        target: None,
        encoding: "cp932".to_string(),
        policy: "relocate_cstr_pool".to_string(),
        choice_group: first.choice_group,
        choice_slot: first.choice_slot,
        body_origin: first.body_origin.clone(),
        pua: pua_metadata(&cstr.text),
        refs: refs.iter().map(make_reference).collect(),
    })
}

fn build_name_entry(
    bundle: &Bundle,
    index: usize,
    record: &TalkInfoRecord,
) -> Result<TranslationEntry, BundleError> {
    let cstr = cstr_entry(bundle, record.name_cstr_id, "TalkInfo 名称")?;
    let reference = TextReference {
        file: record.source.file.clone(),
        file_id: record.source.file_id,
        inst_offset: hex_offset(record.call_offset),
        line: record.source.line,
        dbg_record: record.source.dbg_record,
        type_name: "name".to_string(),
        choice_group: None,
        choice_slot: None,
    };
    Ok(TranslationEntry {
        file: record.source.file.clone(),
        file_id: record.source.file_id,
        index,
        type_name: "name".to_string(),
        scr_name: Some(editable_name(&cstr.text)),
        name: Some(editable_name(&cstr.text)),
        message: cstr.text.clone(),
        scr_msg: cstr.text.clone(),
        cstr_id: record.name_cstr_id,
        name_cstr_id: None,
        talk_info_id: Some(record.talk_info_id),
        talk_style: None,
        message_id: None,
        offset: cstr.pool_offset,
        size: cstr.size.saturating_sub(1),
        cstr_size: cstr.size,
        inst_offset: hex_offset(record.call_offset),
        line: record.source.line,
        dbg_record: record.source.dbg_record,
        opcode: "C6/sysSetTalkInfo$isii".to_string(),
        target: None,
        encoding: "cp932".to_string(),
        policy: "relocate_cstr_pool".to_string(),
        choice_group: None,
        choice_slot: None,
        body_origin: None,
        pua: pua_metadata(&cstr.text),
        refs: vec![reference],
    })
}

fn collect_wrapper_calls(
    bundle: &Bundle,
) -> Vec<(usize, ApiEntry, Vec<crate::bundle::Instruction>)> {
    let mut result = Vec::new();
    for (index, insn) in bundle.instructions.iter().enumerate() {
        if insn.opcode != 0xc6 {
            continue;
        }
        let Some(address) = insn.operand.map(|value| value as usize) else {
            continue;
        };
        let Some(api) = bundle.api_by_address.get(&address) else {
            continue;
        };
        let Some(args) = bundle.argument_window(index, api.argc) else {
            continue;
        };
        result.push((index, api.clone(), args.to_vec()));
    }
    result
}

fn collect_talk_info(
    bundle: &Bundle,
    wrapper_calls: &[(usize, ApiEntry, Vec<crate::bundle::Instruction>)],
) -> Result<HashMap<u32, TalkInfoRecord>, BundleError> {
    let mut records = HashMap::new();
    for (index, api, args) in wrapper_calls {
        if api.name != "sysSetTalkInfo$isii" || args.len() != 4 {
            continue;
        }
        if args[0].opcode != 0x7e
            || args[1].opcode != 0x82
            || args[2].opcode != 0x7e
            || args[3].opcode != 0x7e
        {
            continue;
        }
        let talk_info_id = args[0].operand.unwrap();
        let name_cstr_id = args[1].operand.unwrap() as usize;
        let name = cstr_entry(bundle, name_cstr_id, "TalkInfo")?.text.clone();
        let call_offset = bundle.instructions[*index].offset;
        let source = bundle.source_info(call_offset);
        let record = TalkInfoRecord {
            talk_info_id,
            name_cstr_id,
            name,
            trailing_ints: vec![args[2].operand.unwrap(), args[3].operand.unwrap()],
            call_offset,
            source,
        };
        if let Some(previous) = records.insert(talk_info_id, record.clone()) {
            if previous.name_cstr_id != record.name_cstr_id {
                return Err(format!("TalkInfo ID {} 被冲突地重复定义", talk_info_id));
            }
        }
    }
    if records.is_empty() {
        return Err("没有找到 sysSetTalkInfo$isii 记录".to_string());
    }
    Ok(records)
}

fn add_occurrence(groups: &mut OccurrenceGroups, occurrence: Occurrence) {
    let key = (
        occurrence.group.clone(),
        occurrence.file_id,
        occurrence.cstr_id,
        occurrence.type_name.clone(),
        occurrence.name_cstr_id,
        occurrence.talk_style,
    );
    groups.entry(key).or_default().push(occurrence);
}

pub fn build_plan(bundle: &Bundle) -> Result<ExtractPlan, BundleError> {
    if bundle
        .ftbl1
        .iter()
        .any(|entry| entry.name == "sysSetTalkInfo$isii")
    {
        return build_plan_sinfonia1(bundle);
    }
    if bundle.ftbl1.iter().any(|entry| entry.name == "Mess$is")
        && bundle.ftbl1.iter().any(|entry| entry.name == "Talk$ii")
    {
        return build_plan_sinfonia2(bundle);
    }
    Err("无法识别脚本文本调用结构".to_string())
}

fn build_plan_sinfonia1(bundle: &Bundle) -> Result<ExtractPlan, BundleError> {
    let wrapper_calls = collect_wrapper_calls(bundle);
    let talk_info = collect_talk_info(bundle, &wrapper_calls)?;
    let dialogue_wrapper =
        bundle.find_wrapper_by_apis(&["sysTalkOpen$", "sysTalkDirect$iissii"], 6)?;
    let dialogue_argc = (*bundle
        .function_entries
        .get(&dialogue_wrapper)
        .ok_or_else(|| "对话包装函数没有 B5 标记".to_string())? as usize)
        .saturating_sub(1);
    if dialogue_argc != 6 {
        return Err(format!("对话包装函数参数数目异常: {}", dialogue_argc));
    }
    let select_wrapper = bundle
        .ftbl1
        .iter()
        .find(|entry| entry.name == "sysSelect$iissssss")
        .map(|entry| entry.address)
        .ok_or_else(|| "FTBL_1 缺少 sysSelect$iissssss".to_string())?;
    let mut groups = OccurrenceGroups::new();
    let mut warnings = Vec::new();

    for (index, insn) in bundle.instructions.iter().enumerate() {
        if insn.opcode != 0xb2 || insn.operand != Some(dialogue_wrapper as u32) {
            continue;
        }
        let Some(args) = bundle.argument_window(index, dialogue_argc) else {
            warnings.push(format!("对话调用 0x{:x} 参数不连续", insn.offset));
            continue;
        };
        let source = bundle.source_info(insn.offset);
        let talk_info_id = args[1].operand;
        let Some(talk_info_id) = talk_info_id else {
            warnings.push(format!(
                "对话调用 0x{:x} TalkInfo ID 不是立即数",
                insn.offset
            ));
            continue;
        };
        let Some(record) = talk_info.get(&talk_info_id) else {
            warnings.push(format!(
                "对话调用 0x{:x} 找不到 TalkInfo ID {}",
                insn.offset, talk_info_id
            ));
            continue;
        };
        let aux_is_empty = args[3].opcode == 0x82
            && args[3]
                .operand
                .and_then(|id| bundle.cstr.get(id as usize))
                .is_some_and(|entry| entry.text.is_empty());
        if !aux_is_empty {
            warnings.push(format!("对话调用 0x{:x} 的辅助字符串不是空串", insn.offset));
        }
        let body_index = index - dialogue_argc + 2;
        let body_arg = args[2];
        let (body_cstr_id, body_origin) = if body_arg.opcode == 0x82 {
            (
                body_arg.operand.unwrap() as usize,
                Some("push_cstr".to_string()),
            )
        } else if body_arg.opcode == 0x87 {
            match bundle.resolve_frame_string(body_index) {
                Some((cstr_id, _)) => (cstr_id, Some("frame_str_constant".to_string())),
                None => {
                    warnings.push(format!("对话调用 0x{:x} 的局部正文无法追溯", insn.offset));
                    continue;
                }
            }
        } else {
            warnings.push(format!(
                "对话调用 0x{:x} 正文 opcode 0x{:02x} 未支持",
                insn.offset, body_arg.opcode
            ));
            continue;
        };
        cstr_entry(bundle, body_cstr_id, "对话正文")?;
        let type_name = if record.name.is_empty() {
            "narration"
        } else {
            "dialogue"
        };
        add_occurrence(
            &mut groups,
            Occurrence {
                group: "script".to_string(),
                file_id: source.file_id,
                file: source.file.clone(),
                cstr_id: body_cstr_id,
                type_name: type_name.to_string(),
                speaker_name: None,
                name_cstr_id: Some(record.name_cstr_id),
                talk_info_id: Some(talk_info_id),
                talk_style: None,
                message_id: None,
                body_origin,
                call_offset: insn.offset,
                source,
                choice_group: None,
                choice_slot: None,
            },
        );
    }

    let mut choice_group = 0usize;
    for (index, api, args) in &wrapper_calls {
        if api.address != select_wrapper || api.name != "sysSelect$iissssss" || args.len() != 8 {
            continue;
        }
        let call = bundle.instructions[*index];
        let source = bundle.source_info(call.offset);
        let system = is_system_file(&source.file);
        for slot in 0..6usize {
            let arg = args[2 + slot];
            if arg.opcode != 0x82 {
                warnings.push(format!(
                    "选择调用 0x{:x} 槽 {} 不是 PUSH_STR",
                    call.offset, slot
                ));
                continue;
            }
            let cstr_id = arg.operand.unwrap() as usize;
            let text = cstr_entry(bundle, cstr_id, "选项")?.text.as_str();
            if text.is_empty() {
                continue;
            }
            add_occurrence(
                &mut groups,
                Occurrence {
                    group: if system { "system" } else { "script" }.to_string(),
                    file_id: source.file_id,
                    file: source.file.clone(),
                    cstr_id,
                    type_name: if system { "system_choice" } else { "choice" }.to_string(),
                    speaker_name: None,
                    name_cstr_id: None,
                    talk_info_id: None,
                    talk_style: None,
                    message_id: None,
                    body_origin: None,
                    call_offset: call.offset,
                    source: source.clone(),
                    choice_group: Some(choice_group),
                    choice_slot: Some(slot),
                },
            );
        }
        choice_group += 1;
    }

    finish_plan(bundle, groups, warnings, talk_info.into_values().collect())
}

fn finish_plan(
    bundle: &Bundle,
    groups: OccurrenceGroups,
    warnings: Vec<String>,
    mut names: Vec<TalkInfoRecord>,
) -> Result<ExtractPlan, BundleError> {
    let mut grouped = SourceGroups::new();
    for ((_group, _file_id, _cstr_id, _type_name, _name_cstr_id, _talk_style), mut refs) in groups {
        refs.sort_by_key(|occurrence| occurrence.call_offset);
        let first = refs[0].clone();
        grouped
            .entry((first.group.clone(), first.file_id))
            .or_default()
            .push((first, refs));
    }

    let mut planned = Vec::new();
    let mut report = ExtractReport {
        scanned_files: bundle.source_files.len(),
        warnings: warnings.len(),
        ..ExtractReport::default()
    };
    for ((group, file_id), mut entries) in grouped {
        entries.sort_by(|(left, _), (right, _)| {
            left.call_offset
                .cmp(&right.call_offset)
                .then_with(|| left.choice_group.cmp(&right.choice_group))
                .then_with(|| left.choice_slot.cmp(&right.choice_slot))
                .then_with(|| left.cstr_id.cmp(&right.cstr_id))
                .then_with(|| left.type_name.cmp(&right.type_name))
                .then_with(|| left.name_cstr_id.cmp(&right.name_cstr_id))
        });
        let file = bundle.source_files[file_id].clone();
        let mut json_entries = Vec::new();
        for (index, (first, refs)) in entries.iter().enumerate() {
            let entry = build_entry(bundle, index, first, refs)?;
            match entry.type_name.as_str() {
                "dialogue" | "narration" => report.dialogue_entries += 1,
                "choice" | "system_choice" => report.choice_entries += 1,
                _ => {}
            }
            json_entries.push(entry);
        }
        let relative = format!(
            "translation_json/{}/{}__{}.json",
            if group == "system" {
                "system"
            } else {
                "scripts"
            },
            file_id,
            sanitize_file_component(&file)
        );
        report.extracted_entries += json_entries.len();
        planned.push(PlannedTranslationFile {
            relative_path: relative,
            file_id: Some(file_id),
            file: Some(file.clone()),
            group: group.clone(),
            translation: TranslationFile {
                format: TRANSLATION_FORMAT.to_string(),
                version: 1,
                group: group.clone(),
                file_id: Some(file_id),
                file: Some(file),
                entries: json_entries,
            },
        });
    }

    for file_id in 0..bundle.source_files.len() {
        if !planned.iter().any(|item| item.file_id == Some(file_id)) {
            let file = bundle.source_files[file_id].clone();
            let relative = format!(
                "translation_json/scripts/{}__{}.json",
                file_id,
                sanitize_file_component(&file)
            );
            planned.push(PlannedTranslationFile {
                relative_path: relative,
                file_id: Some(file_id),
                file: Some(file.clone()),
                group: "script".to_string(),
                translation: TranslationFile {
                    format: TRANSLATION_FORMAT.to_string(),
                    version: 1,
                    group: "script".to_string(),
                    file_id: Some(file_id),
                    file: Some(file),
                    entries: Vec::new(),
                },
            });
        }
    }

    names.sort_by_key(|record| record.talk_info_id);
    let mut name_entries = Vec::new();
    for (index, record) in names.iter().enumerate() {
        name_entries.push(build_name_entry(bundle, index, record)?);
    }
    report.name_entries = name_entries.len();
    report.extracted_entries += name_entries.len();
    planned.push(PlannedTranslationFile {
        relative_path: "translation_json/names.json".to_string(),
        file_id: None,
        file: None,
        group: "names".to_string(),
        translation: TranslationFile {
            format: TRANSLATION_FORMAT.to_string(),
            version: 1,
            group: "names".to_string(),
            file_id: None,
            file: None,
            entries: name_entries,
        },
    });
    planned.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    report.json_files = planned.len();
    Ok(ExtractPlan {
        translation_files: planned,
        warnings,
        report,
    })
}

fn api_address(bundle: &Bundle, name: &str) -> Result<usize, BundleError> {
    bundle
        .ftbl1
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.address)
        .ok_or_else(|| format!("FTBL_1 缺少 {}", name))
}

fn sinfonia2_speaker_name(style: Option<[u32; 2]>) -> &'static str {
    match style {
        Some([0, 0]) => "アリス",
        Some([1, 1]) => "ションマオ",
        Some([2, 2]) => "うるし",
        Some([3, 3]) => "マリオン",
        Some([4, 4]) => "ミリオン",
        Some([5, 5]) => "ミリア",
        Some([6, 14 | 15]) => "シルフィーネ",
        Some([7, 17]) => "アスカロン",
        Some([12, 21]) => "ガリオン",
        Some([14, 9]) => "ミルフィ",
        Some([15, 10]) => "ミオリ",
        Some([16, 11]) => "ノエル",
        Some([17, 12]) => "リュミエール",
        Some([19, 7]) => "ジョルジュ",
        Some([20, 8]) => "クロウフォード",
        Some([21, 6]) => "ウィル",
        Some([24, 84]) => "鱗姫",
        Some([25, 25]) => "アル・ウード",
        Some([26, 45]) => "ユニコーン",
        Some([27, 32]) => "ディゼルⅦ",
        Some([28, 36]) => "ビフレスト",
        Some([32, 37]) => "フーリン",
        _ => "",
    }
}

fn build_plan_sinfonia2(bundle: &Bundle) -> Result<ExtractPlan, BundleError> {
    let mess = api_address(bundle, "Mess$is")?;
    let talk = api_address(bundle, "Talk$ii")?;
    let select = api_address(bundle, "Select$i")?;
    let select_clear = api_address(bundle, "SelectClr$i")?;
    let mut command_slots = HashMap::new();
    for api in &bundle.ftbl1 {
        let Some(number) = api
            .name
            .strip_prefix("Cmd")
            .and_then(|value| value.strip_suffix("$s"))
            .and_then(|value| value.parse::<usize>().ok())
        else {
            continue;
        };
        if (1..=9).contains(&number) {
            command_slots.insert(api.address, number - 1);
        }
    }

    let mut groups = OccurrenceGroups::new();
    let mut warnings = Vec::new();
    let mut current_style = None;
    let mut pending_choices = BTreeMap::<usize, Occurrence>::new();
    let mut choice_group = 0usize;
    let mut messages_without_style = 0usize;

    for (index, insn) in bundle.instructions.iter().enumerate() {
        if insn.opcode != 0xb2 {
            continue;
        }
        let Some(address) = insn.operand.map(|value| value as usize) else {
            continue;
        };
        if address == talk {
            let Some(args) = bundle.argument_window(index, 2) else {
                warnings.push(format!("Talk$ii 调用 0x{:x} 参数不连续", insn.offset));
                continue;
            };
            if args.iter().all(|arg| arg.opcode == 0x7e) {
                current_style = Some([args[0].operand.unwrap(), args[1].operand.unwrap()]);
            } else {
                current_style = None;
                warnings.push(format!("Talk$ii 调用 0x{:x} 参数不是立即数", insn.offset));
            }
            continue;
        }
        if address == mess {
            let Some(args) = bundle.argument_window(index, 2) else {
                warnings.push(format!("Mess$is 调用 0x{:x} 参数不连续", insn.offset));
                continue;
            };
            if args[0].opcode != 0x7e || args[1].opcode != 0x82 {
                warnings.push(format!("Mess$is 调用 0x{:x} 参数类型异常", insn.offset));
                continue;
            }
            let message_id = args[0].operand.unwrap();
            let cstr_id = args[1].operand.unwrap() as usize;
            cstr_entry(bundle, cstr_id, "Mess$is 正文")?;
            if current_style.is_none() {
                messages_without_style += 1;
            }
            let source = bundle.source_info(insn.offset);
            let type_name = if current_style == Some([33, 94]) {
                "narration"
            } else {
                "dialogue"
            };
            add_occurrence(
                &mut groups,
                Occurrence {
                    group: "script".to_string(),
                    file_id: source.file_id,
                    file: source.file.clone(),
                    cstr_id,
                    type_name: type_name.to_string(),
                    speaker_name: Some(sinfonia2_speaker_name(current_style).to_string()),
                    name_cstr_id: None,
                    talk_info_id: None,
                    talk_style: current_style,
                    message_id: Some(message_id),
                    body_origin: Some("b2_mess_is".to_string()),
                    call_offset: insn.offset,
                    source,
                    choice_group: None,
                    choice_slot: None,
                },
            );
            continue;
        }
        if address == select_clear {
            pending_choices.clear();
            continue;
        }
        if let Some(&slot) = command_slots.get(&address) {
            let Some(args) = bundle.argument_window(index, 1) else {
                warnings.push(format!(
                    "Cmd{} 调用 0x{:x} 参数不连续",
                    slot + 1,
                    insn.offset
                ));
                continue;
            };
            if args[0].opcode != 0x82 {
                warnings.push(format!(
                    "Cmd{} 调用 0x{:x} 参数不是字符串",
                    slot + 1,
                    insn.offset
                ));
                continue;
            }
            let cstr_id = args[0].operand.unwrap() as usize;
            if cstr_entry(bundle, cstr_id, "选项")?.text.is_empty() {
                pending_choices.remove(&slot);
                continue;
            }
            let source = bundle.source_info(insn.offset);
            pending_choices.insert(
                slot,
                Occurrence {
                    group: if is_system_file(&source.file) {
                        "system".to_string()
                    } else {
                        "script".to_string()
                    },
                    file_id: source.file_id,
                    file: source.file.clone(),
                    cstr_id,
                    type_name: if is_system_file(&source.file) {
                        "system_choice".to_string()
                    } else {
                        "choice".to_string()
                    },
                    speaker_name: None,
                    name_cstr_id: None,
                    talk_info_id: None,
                    talk_style: None,
                    message_id: None,
                    body_origin: Some(format!("b2_cmd{}", slot + 1)),
                    call_offset: insn.offset,
                    source,
                    choice_group: Some(choice_group),
                    choice_slot: Some(slot),
                },
            );
            continue;
        }
        if address == select {
            for (_, mut occurrence) in std::mem::take(&mut pending_choices) {
                occurrence.choice_group = Some(choice_group);
                add_occurrence(&mut groups, occurrence);
            }
            choice_group += 1;
        }
    }
    if messages_without_style > 0 {
        warnings.push(format!(
            "{} 条 Mess$is 正文之前没有可用的 Talk$ii 样式",
            messages_without_style
        ));
    }
    if !pending_choices.is_empty() {
        warnings.push(format!(
            "脚本末尾有 {} 个未提交到 Select$i 的选项",
            pending_choices.len()
        ));
    }
    finish_plan(bundle, groups, warnings, Vec::new())
}

pub fn sanitize_file_component(file: &str) -> String {
    let mut result = file
        .chars()
        .map(|character| {
            if character.is_control() || "<>:\"/\\|?*".contains(character) {
                '_'
            } else {
                character
            }
        })
        .collect::<String>();
    while result.ends_with('.') || result.ends_with(' ') {
        result.pop();
    }
    if result.is_empty() {
        "unnamed".to_string()
    } else {
        result
    }
}

pub fn extract_workspace(options: &ExtractOptions) -> Result<ExtractReport, BundleError> {
    let bundle = Bundle::load(&options.input)?;
    let plan = build_plan(&bundle)?;
    crate::workspace::write_extracted_workspace(&bundle, &plan, options)
}

#[cfg(test)]
mod tests {
    use super::sanitize_file_component;

    #[test]
    fn sanitizes_only_windows_forbidden_filename_content() {
        assert_eq!(sanitize_file_component("a<b>:c.ss. "), "a_b__c.ss");
        assert_eq!(sanitize_file_component("日本語.ss"), "日本語.ss");
        assert_eq!(sanitize_file_component("<>:*?"), "_____");
    }
}
