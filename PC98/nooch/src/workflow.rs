use crate::codec::{
    encode_translation, expand12, expand3, find_nooch3_messages, find_text_spans, load_nooch1_map,
    load_nooch2_map, load_nooch3_map, scan_exe_text_slots, serialize_expanded, serialize_nooch2,
    strip_tokens,
};
use crate::discovery::{
    basename, discover, disk_directory, safe_join, sha256_hex, slash_path, DiskSource,
};
use crate::model::{
    DiskManifest, FileManifest, GameId, GameManifest, TranslationEntry, TranslationFileManifest,
    WorkspaceManifest, LEGACY_WORKSPACE_FORMAT, WORKSPACE_FORMAT,
};
use crate::nooch3::SceneArchive;
use anyhow::{bail, Context, Result};
use drrnger_d88_tool::font::{normalize_character, prepare_font, EncodingPlan, FontPatchRequest};
use pc98_fdi_unpack::{extract_fdi_files, rebuild_fdi, FdiFileData};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub struct ExtractReport {
    pub disks: usize,
    pub files: usize,
    pub messages: usize,
}

#[derive(Debug)]
pub struct InjectReport {
    pub disks: usize,
    pub changed_files: usize,
    pub changed_messages: usize,
    pub patched_glyphs: usize,
}

pub fn extract(input: &Path, workspace: &Path, overwrite: bool) -> Result<ExtractReport> {
    let previous = read_previous_translations(workspace)?;
    prepare_destination(workspace, overwrite)?;
    let temporary = temporary_sibling(workspace)?;
    clear_temporary(&temporary)?;
    fs::create_dir_all(&temporary)
        .with_context(|| format!("创建临时工作目录 {} 失败", temporary.display()))?;

    let result = (|| {
        let groups = discover(input, workspace)?;
        let mut games = Vec::new();
        let mut total_files = 0usize;
        let mut total_messages = 0usize;
        for game in GameId::ALL {
            let disks = groups.get(&game).expect("discover checks all games");
            let game_root = temporary.join("games").join(game.as_str());
            let files_root = game_root.join("files");
            fs::create_dir_all(&files_root)?;
            let map = load_game_map(game, disks)?;
            let mut disk_manifests = Vec::with_capacity(disks.len());
            for (disk_index, disk) in disks.iter().enumerate() {
                let disk_dir = disk_directory(disk_index, &disk.path);
                let output_dir = files_root.join(&disk_dir);
                fs::create_dir_all(&output_dir)?;
                let mut files = Vec::with_capacity(disk.files.len());
                for file in &disk.files {
                    let target = safe_join(&output_dir, &file.path)?;
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&target, &file.data)
                        .with_context(|| format!("写入真实文件 {} 失败", target.display()))?;
                    files.push(FileManifest {
                        path: file.path.replace('\\', "/"),
                        size: file.data.len(),
                        sha256: sha256_hex(&file.data),
                    });
                    total_files += 1;
                }
                disk_manifests.push(DiskManifest {
                    disk_index,
                    source_relative: disk.relative.clone(),
                    source_file: disk
                        .path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    source_sha256: disk.sha256.clone(),
                    extracted_directory: format!("games/{}/files/{disk_dir}", game.as_str()),
                    files,
                });
            }
            let mut entries = extract_game_entries(game, disks, &map)?;
            if let Some(old) = previous.get(&game) {
                merge_existing_translations(&mut entries, old);
            }
            total_messages += entries.len();
            let translations = write_translation_files(&game_root, game, entries)?;
            games.push(GameManifest {
                game,
                messages: None,
                translations,
                disks: disk_manifests,
            });
        }
        let manifest = WorkspaceManifest {
            _format: WORKSPACE_FORMAT.to_string(),
            tool_version: TOOL_VERSION.to_string(),
            input_root: slash_path(input),
            games,
        };
        write_json(&temporary.join("workspace.json"), &manifest)?;
        fs::write(
            temporary.join("README.txt"),
            "Nooch 工具翻译工作区\r\n\r\n每个原资源对应 games/<作品>/json/ 下的一个同名 JSON。\r\n只编辑各 JSON 中的 message 和 name。\r\n下划线字段、scr_msg 均用于定位与校验，请勿修改。\r\nname=null 表示删除姓名框；<PLAYER_NAME> 表示运行时玩家名。\r\n_kind=exe_string 是 EXE 内嵌定长文本，编码后不能超过 _capacity。\r\n文本框会自动换行；不要在 message 内直接加入 CR/LF。\r\n",
        )?;
        Ok(ExtractReport {
            disks: manifest.games.iter().map(|game| game.disks.len()).sum(),
            files: total_files,
            messages: total_messages,
        })
    })();
    match result {
        Ok(report) => {
            commit_destination(&temporary, workspace, overwrite)?;
            Ok(report)
        }
        Err(error) => {
            let _ = fs::remove_dir_all(&temporary);
            Err(error)
        }
    }
}

pub fn inject(
    input: &Path,
    workspace: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<InjectReport> {
    if same_path(input, output)? || same_path(workspace, output)? {
        bail!("输出目录不能与输入根目录或翻译工作区相同");
    }
    prepare_destination(output, overwrite)?;
    let manifest: WorkspaceManifest = read_json(&workspace.join("workspace.json"))?;
    if manifest._format != WORKSPACE_FORMAT && manifest._format != LEGACY_WORKSPACE_FORMAT {
        bail!("不支持的工作区格式 {:?}", manifest._format);
    }
    let temporary = temporary_sibling(output)?;
    clear_temporary(&temporary)?;
    fs::create_dir_all(&temporary)?;

    let result = (|| {
        let mut disk_total = 0usize;
        let mut changed_files = 0usize;
        let mut changed_messages = 0usize;
        let mut patched_glyphs = 0usize;
        for game_id in GameId::ALL {
            let game = manifest
                .games
                .iter()
                .find(|game| game.game == game_id)
                .with_context(|| format!("工作区缺少 {}", game_id.as_str()))?;
            let entries = load_translation_entries(workspace, game, game_id)?;
            changed_messages += entries.iter().filter(|entry| is_changed(entry)).count();

            let plan = build_encoding_plan(game_id, &entries)?;
            let needed_glyphs = changed_output_characters(&entries)?;
            let requests = plan
                .requests()
                .into_iter()
                .filter(|request| {
                    request.carrier != request.replacement
                        && needed_glyphs.contains(&request.replacement)
                })
                .collect::<Vec<FontPatchRequest>>();
            let font = prepare_font(&requests, &BTreeSet::new()).map_err(anyhow::Error::msg)?;
            patched_glyphs += font.patched_glyphs;
            let font_root = temporary.join(game_id.as_str());
            fs::create_dir_all(&font_root)?;
            fs::write(font_root.join("font.tmp"), &font.bytes)?;
            fs::write(font_root.join("font.bmp"), &font.bytes)?;
            write_json(
                &font_root.join("font_map.json"),
                &plan.manifest_entries().map_err(anyhow::Error::msg)?,
            )?;

            let sources = load_source_disks(input, game)?;
            let source_map = load_map_from_source_files(game_id, &sources)?;
            let encoding_map = if game_id == GameId::Nooch2 {
                build_nooch2_translation_map(&entries, &plan, &source_map)?
            } else {
                source_map.clone()
            };
            let mut entries_by_disk = HashMap::<String, Vec<&TranslationEntry>>::new();
            for entry in &entries {
                entries_by_disk
                    .entry(entry._disk_sha256.clone())
                    .or_default()
                    .push(entry);
            }
            for source in sources {
                disk_total += 1;
                let disk_entries = entries_by_disk
                    .get(&source.manifest.source_sha256)
                    .cloned()
                    .unwrap_or_default();
                let mut by_member = HashMap::<String, Vec<&TranslationEntry>>::new();
                for entry in disk_entries {
                    by_member
                        .entry(entry._member.to_ascii_uppercase())
                        .or_default()
                        .push(entry);
                }
                let mut replacements = BTreeMap::new();
                for (member_key, member_entries) in by_member {
                    let file = source
                        .files
                        .iter()
                        .find(|file| file.path.eq_ignore_ascii_case(&member_key))
                        .or_else(|| {
                            source
                                .files
                                .iter()
                                .find(|file| file.path.to_ascii_uppercase() == member_key)
                        })
                        .with_context(|| {
                            format!(
                                "{} 中不存在资源 {}",
                                source.manifest.source_file, member_key
                            )
                        })?;
                    let kinds = member_entries
                        .iter()
                        .map(|entry| entry._kind.as_str())
                        .collect::<HashSet<_>>();
                    let rebuilt = if kinds == HashSet::from(["exe_string"]) {
                        patch_exe_strings(game_id, &file.data, &member_entries, &plan)?
                    } else if kinds == HashSet::from(["message"]) {
                        match game_id {
                            GameId::Nooch | GameId::Nooch2 => patch_nooch12(
                                game_id,
                                &file.data,
                                &member_entries,
                                &source_map,
                                &encoding_map,
                                &plan,
                                game_id == GameId::Nooch2,
                            )?,
                            GameId::Nooch3 => {
                                patch_nooch3(&file.data, &member_entries, &source_map, &plan)?
                            }
                        }
                    } else {
                        bail!("{} 同时出现不兼容的条目类型", member_key);
                    };
                    if rebuilt != file.data {
                        replacements.insert(file.path.clone(), rebuilt);
                        changed_files += 1;
                    }
                }
                if game_id == GameId::Nooch2 {
                    for file in &source.files {
                        let file_name = basename(&file.path);
                        if file_name.eq_ignore_ascii_case("SYSTEM.MAC") {
                            let rebuilt = rewrite_nooch2_map(&file.data, &encoding_map)?;
                            if rebuilt != file.data
                                && replacements.insert(file.path.clone(), rebuilt).is_none()
                            {
                                changed_files += 1;
                            }
                        } else if file_name.to_ascii_uppercase().starts_with("SEEN")
                            && file_name.to_ascii_uppercase().ends_with(".TXT")
                            && !replacements.contains_key(&file.path)
                        {
                            let rebuilt = patch_nooch12(
                                game_id,
                                &file.data,
                                &[],
                                &source_map,
                                &encoding_map,
                                &plan,
                                true,
                            )?;
                            if rebuilt != file.data {
                                replacements.insert(file.path.clone(), rebuilt);
                                changed_files += 1;
                            }
                        }
                    }
                }
                let size_changes = replacements
                    .iter()
                    .filter_map(|(path, data)| {
                        source
                            .files
                            .iter()
                            .find(|file| file.path.eq_ignore_ascii_case(path))
                            .map(|file| {
                                format!(
                                    "{} {}->{}, {:+}",
                                    path,
                                    file.data.len(),
                                    data.len(),
                                    data.len() as i64 - file.data.len() as i64
                                )
                            })
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                let packed =
                    rebuild_fdi(&source.bytes, &source.path.to_string_lossy(), &replacements)
                        .map_err(anyhow::Error::msg)
                        .with_context(|| {
                            format!(
                                "重建 {} 失败；变更文件尺寸: {}",
                                source.manifest.source_relative, size_changes
                            )
                        })?;
                let target = safe_join(&temporary.join("fdi"), &source.manifest.source_relative)?;
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&target, packed.bytes)
                    .with_context(|| format!("写入重建 FDI {} 失败", target.display()))?;
            }
        }
        fs::write(
            temporary.join("README.txt"),
            "fdi/ 下是保持原目录结构的完整重建盘组。\r\nnooch、nooch2、nooch3 下的 font.bmp/font.tmp 为三作各自字库；两个文件内容相同，font.bmp 可直接供 Neko Project 使用。\r\n原始 FDI 从未被修改。\r\n",
        )?;
        Ok(InjectReport {
            disks: disk_total,
            changed_files,
            changed_messages,
            patched_glyphs,
        })
    })();
    match result {
        Ok(report) => {
            commit_destination(&temporary, output, overwrite)?;
            Ok(report)
        }
        Err(error) => {
            let _ = fs::remove_dir_all(&temporary);
            Err(error)
        }
    }
}

fn extract_game_entries(
    game: GameId,
    disks: &[DiskSource],
    map: &[Vec<u8>],
) -> Result<Vec<TranslationEntry>> {
    let mut entries = Vec::new();
    for disk in disks {
        for file in &disk.files {
            let file_name = basename(&file.path).to_ascii_uppercase();
            match game {
                GameId::Nooch | GameId::Nooch2
                    if file_name.starts_with("SEEN") && file_name.ends_with(".TXT") =>
                {
                    // SEEN*.TXT begins with scene-selection expressions and a binary
                    // lookup table.  Those bytes use values that overlap the compact
                    // kana table, so expanding the whole file produces fake strings
                    // such as `ら0001102ひ`.  The executable enters the actual text
                    // script at the first `Ixxx\n` scene marker.
                    let script_start = find_seen_script_start(&file.data)
                        .with_context(|| format!("{} 缺少 Ixxx 脚本起点", file.path))?;
                    let first_line_index = file.data[..script_start]
                        .iter()
                        .filter(|&&byte| byte == b'\n')
                        .count();
                    let mut line_start = script_start;
                    for (relative_line_index, line_with_lf) in file.data[script_start..]
                        .split_inclusive(|&b| b == b'\n')
                        .enumerate()
                    {
                        let line_index = first_line_index + relative_line_index;
                        let line = line_with_lf.strip_suffix(b"\n").unwrap_or(line_with_lf);
                        let expanded = expand12(game, line, map);
                        for (span_index, span) in find_text_spans(&expanded).into_iter().enumerate()
                        {
                            let index = entries.len();
                            entries.push(TranslationEntry {
                                _file: file.path.replace('\\', "/"),
                                _index: index,
                                _kind: "message".to_string(),
                                _ref: line_start + span.start,
                                _code_word: span.start,
                                name: span.name.clone(),
                                _scr_name: span.name,
                                scr_msg: span.text.clone(),
                                message: span.text,
                                _disk_sha256: disk.sha256.clone(),
                                _member: file.path.clone(),
                                _line: Some(line_index),
                                _span: Some(span_index),
                                _scene: None,
                                _record: None,
                                _script: None,
                                _message: None,
                                _category: span.command_text.then(|| "command_text".to_string()),
                                _slot_offset: None,
                                _capacity: None,
                            });
                        }
                        line_start += line_with_lf.len();
                    }
                }
                GameId::Nooch3 if file_name == "SEEN_A.TXT" => {
                    let archive = SceneArchive::parse(&file.data)?;
                    for (record_index, record) in archive.records.iter().enumerate() {
                        for field in &record.scripts {
                            let expanded =
                                expand3(&record.unpacked[field.data_start..field.data_end], map);
                            for (message_index, span) in
                                find_nooch3_messages(&expanded).into_iter().enumerate()
                            {
                                let index = entries.len();
                                entries.push(TranslationEntry {
                                    _file: file.path.replace('\\', "/"),
                                    _index: index,
                                    _kind: "message".to_string(),
                                    _ref: record_index,
                                    _code_word: span.start,
                                    name: span.name.clone(),
                                    _scr_name: span.name,
                                    scr_msg: span.text.clone(),
                                    message: span.text,
                                    _disk_sha256: disk.sha256.clone(),
                                    _member: file.path.clone(),
                                    _line: None,
                                    _span: None,
                                    _scene: Some(record.id),
                                    _record: Some(record_index),
                                    _script: Some(field.index),
                                    _message: Some(message_index),
                                    _category: Some(field.category.to_string()),
                                    _slot_offset: None,
                                    _capacity: None,
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
            if file_name == main_exe_name(game) {
                for slot in scan_exe_text_slots(&file.data) {
                    for (span_index, span) in slot.spans.into_iter().enumerate() {
                        let index = entries.len();
                        entries.push(TranslationEntry {
                            _file: file.path.replace('\\', "/"),
                            _index: index,
                            _kind: "exe_string".to_string(),
                            _ref: slot.offset + span.start,
                            _code_word: span.start,
                            name: span.name.clone(),
                            _scr_name: span.name,
                            scr_msg: span.text.clone(),
                            message: span.text,
                            _disk_sha256: disk.sha256.clone(),
                            _member: file.path.clone(),
                            _line: None,
                            _span: Some(span_index),
                            _scene: None,
                            _record: None,
                            _script: None,
                            _message: None,
                            _category: Some("exe_fixed_string".to_string()),
                            _slot_offset: Some(slot.offset),
                            _capacity: Some(slot.capacity),
                        });
                    }
                }
            }
        }
    }
    Ok(entries)
}

fn find_seen_script_start(data: &[u8]) -> Option<usize> {
    data.windows(5)
        .position(|window| {
            window[0] == b'I' && window[1..4].iter().all(u8::is_ascii_digit) && window[4] == b'\n'
        })
        .map(|marker| marker + 5)
}

fn write_translation_files(
    game_root: &Path,
    game: GameId,
    entries: Vec<TranslationEntry>,
) -> Result<Vec<TranslationFileManifest>> {
    let mut groups = BTreeMap::<(String, String), Vec<TranslationEntry>>::new();
    for entry in entries {
        groups
            .entry((
                entry._member.to_ascii_uppercase(),
                entry._disk_sha256.clone(),
            ))
            .or_default()
            .push(entry);
    }
    let mut base_counts = HashMap::<String, usize>::new();
    for values in groups.values() {
        let base = json_stem(&values[0]._member);
        *base_counts.entry(base.to_ascii_uppercase()).or_default() += 1;
    }
    let json_root = game_root.join("json");
    fs::create_dir_all(&json_root)?;
    let mut used_names = HashSet::new();
    let mut result = Vec::with_capacity(groups.len());
    for ((_member_key, disk_hash), mut values) in groups {
        let member = values[0]._member.clone();
        let base = json_stem(&member);
        let duplicated = base_counts
            .get(&base.to_ascii_uppercase())
            .copied()
            .unwrap_or(0)
            > 1;
        let short_hash = &disk_hash[..disk_hash.len().min(8)];
        let mut file_name = if duplicated {
            format!("{base}__{short_hash}.json")
        } else {
            format!("{base}.json")
        };
        let mut serial = 2usize;
        while !used_names.insert(file_name.to_ascii_uppercase()) {
            file_name = format!("{base}__{short_hash}_{serial}.json");
            serial += 1;
        }
        values.sort_by_key(|entry| entry._index);
        for (index, entry) in values.iter_mut().enumerate() {
            entry._index = index;
            entry._file = member.replace('\\', "/");
        }
        write_json(&json_root.join(&file_name), &values)?;
        result.push(TranslationFileManifest {
            source_file: member.replace('\\', "/"),
            source_disk_sha256: disk_hash,
            member,
            json: format!("games/{}/json/{file_name}", game.as_str()),
            entries: values.len(),
        });
    }
    result.sort_by(|left, right| left.json.cmp(&right.json));
    Ok(result)
}

fn json_stem(member: &str) -> String {
    let stem = Path::new(basename(member))
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let cleaned = stem
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if cleaned.is_empty() {
        "resource".to_string()
    } else {
        cleaned
    }
}

fn patch_nooch12(
    game: GameId,
    data: &[u8],
    entries: &[&TranslationEntry],
    source_map: &[Vec<u8>],
    encoding_map: &[Vec<u8>],
    plan: &EncodingPlan,
    recode_all: bool,
) -> Result<Vec<u8>> {
    let mut by_line = BTreeMap::<usize, Vec<&TranslationEntry>>::new();
    for entry in entries {
        by_line
            .entry(entry._line.context("一、二代条目缺少 _line")?)
            .or_default()
            .push(*entry);
    }
    let first_script_line = recode_all
        .then(|| find_seen_script_start(data))
        .flatten()
        .map(|start| data[..start].iter().filter(|&&byte| byte == b'\n').count());
    let mut output = Vec::with_capacity(data.len());
    for (line_index, line_with_lf) in data.split_inclusive(|&b| b == b'\n').enumerate() {
        let line_entries = by_line.remove(&line_index).unwrap_or_default();
        let recode_line = first_script_line.is_some_and(|first| line_index >= first);
        if line_entries.is_empty() && !recode_line {
            output.extend_from_slice(line_with_lf);
            continue;
        }
        let had_lf = line_with_lf.ends_with(b"\n");
        let line = line_with_lf.strip_suffix(b"\n").unwrap_or(line_with_lf);
        let expanded = expand12(game, line, source_map);
        let spans = find_text_spans(&expanded);
        let mut edits = Vec::new();
        for entry in line_entries {
            let span_index = entry._span.context("一、二代条目缺少 _span")?;
            let span = spans.get(span_index).with_context(|| {
                format!(
                    "{} 第 {} 行不存在文本段 {}",
                    entry._member, line_index, span_index
                )
            })?;
            validate_source_entry(entry, &span.text, &span.name)?;
            if is_changed(entry) {
                edits.push((span.start, span.end, encode_full_entry(entry, plan)?));
            }
        }
        if edits.is_empty() && !recode_line {
            output.extend_from_slice(line_with_lf);
            continue;
        }
        let mut patched = expanded;
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
        for (start, end, replacement) in edits {
            patched.splice(start..end, replacement);
        }
        // Nooch 1 accepts the expanded Shift-JIS stream directly. Recompressing
        // an entire script line through TEXT.MAP / packed-SJIS is unnecessary
        // and can change token boundaries in ways the real interpreter handles
        // differently from our round-trip decoder. Keep translated lines literal
        // so every carrier byte reaches the font ROM unchanged.
        if game == GameId::Nooch {
            output.extend_from_slice(&serialize_expanded(game, &patched)?);
        } else {
            output.extend_from_slice(&serialize_nooch2(&patched, encoding_map)?);
        }
        if had_lf {
            output.push(b'\n');
        }
    }
    if !by_line.is_empty() {
        bail!("JSON 引用了 {} 中不存在的行", entries[0]._member);
    }
    Ok(output)
}

fn build_nooch2_translation_map(
    entries: &[TranslationEntry],
    plan: &EncodingPlan,
    source_map: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>> {
    let mut counts = HashMap::<Vec<u8>, usize>::new();
    for entry in entries.iter().filter(|entry| entry._kind == "message") {
        let encoded = encode_full_entry(entry, plan)?;
        let mut pos = 0usize;
        while pos < encoded.len() {
            let run_start = pos;
            while pos + 1 < encoded.len() && is_sjis_pair(encoded[pos], encoded[pos + 1]) {
                pos += 2;
            }
            let glyphs = (pos - run_start) / 2;
            for width in 2..=8.min(glyphs) {
                for offset in 0..=glyphs - width {
                    let start = run_start + offset * 2;
                    let end = start + width * 2;
                    *counts.entry(encoded[start..end].to_vec()).or_default() += 1;
                }
            }
            if pos == run_start {
                pos += 1;
            }
        }
    }

    let mut candidates = counts
        .into_iter()
        .filter(|(_, occurrences)| *occurrences >= 2)
        .collect::<Vec<_>>();
    candidates.sort_by(|(left, left_count), (right, right_count)| {
        let left_score = (left.len() - 2) * left_count;
        let right_score = (right.len() - 2) * right_count;
        right_score
            .cmp(&left_score)
            .then_with(|| right.len().cmp(&left.len()))
            .then_with(|| right_count.cmp(left_count))
            .then_with(|| left.cmp(right))
    });

    let mut result = candidates
        .into_iter()
        .map(|(value, _)| value)
        .take(source_map.len())
        .collect::<Vec<_>>();
    if result.len() < source_map.len() {
        result.extend(source_map[result.len()..].iter().cloned());
    }
    Ok(result)
}

fn rewrite_nooch2_map(data: &[u8], map: &[Vec<u8>]) -> Result<Vec<u8>> {
    let marker = b"#MAP={\r\n";
    let start = crate::codec::find_bytes(data, marker, 0).context("SYSTEM.MAC 缺少 #MAP 块")?
        + marker.len();
    let end =
        crate::codec::find_bytes(data, b"};", start).context("SYSTEM.MAC 的 #MAP 块未闭合")?;
    let mut output = Vec::with_capacity(data.len());
    output.extend_from_slice(&data[..start]);
    for value in map {
        if value.contains(&b'\r') || value.contains(&b'\n') || value.contains(&0) {
            bail!("二代压缩词典候选含有非法控制字节");
        }
        output.extend_from_slice(value);
        output.extend_from_slice(b"\r\n");
    }
    output.extend_from_slice(&data[end..]);
    Ok(output)
}

fn is_sjis_pair(lead: u8, trail: u8) -> bool {
    ((0x81..=0x9f).contains(&lead) || (0xe0..=0xfc).contains(&lead))
        && ((0x40..=0x7e).contains(&trail) || (0x80..=0xfc).contains(&trail))
}

fn patch_nooch3(
    data: &[u8],
    entries: &[&TranslationEntry],
    map: &[Vec<u8>],
    plan: &EncodingPlan,
) -> Result<Vec<u8>> {
    let archive = SceneArchive::parse(data)?;
    let mut by_script = BTreeMap::<(usize, usize), Vec<&TranslationEntry>>::new();
    for entry in entries {
        by_script
            .entry((
                entry._record.context("三代条目缺少 _record")?,
                entry._script.context("三代条目缺少 _script")?,
            ))
            .or_default()
            .push(*entry);
    }
    let mut changes = BTreeMap::<usize, BTreeMap<usize, Vec<u8>>>::new();
    for ((record_index, script_index), script_entries) in by_script {
        let record = archive
            .records
            .get(record_index)
            .context("三代记录索引越界")?;
        let field = record
            .scripts
            .get(script_index)
            .context("三代脚本索引越界")?;
        let mut expanded = expand3(&record.unpacked[field.data_start..field.data_end], map);
        let spans = find_nooch3_messages(&expanded);
        let mut edits = Vec::new();
        for entry in script_entries {
            if entry._scene != Some(record.id) || entry._category.as_deref() != Some(field.category)
            {
                bail!("{} 的场景或脚本类型元数据已被修改", entry._file);
            }
            let message_index = entry._message.context("三代条目缺少 _message")?;
            let span = spans.get(message_index).context("三代消息索引越界")?;
            validate_source_entry(entry, &span.text, &span.name)?;
            if is_changed(entry) {
                edits.push((span.start, span.end, encode_full_entry(entry, plan)?));
            }
        }
        if edits.is_empty() {
            continue;
        }
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
        for (start, end, replacement) in edits {
            expanded.splice(start..end, replacement);
        }
        let raw = serialize_expanded(GameId::Nooch3, &expanded)?;
        changes
            .entry(record_index)
            .or_default()
            .insert(script_index, raw);
    }
    if changes.is_empty() {
        Ok(data.to_vec())
    } else {
        archive.rebuild(&changes)
    }
}

fn patch_exe_strings(
    game: GameId,
    data: &[u8],
    entries: &[&TranslationEntry],
    plan: &EncodingPlan,
) -> Result<Vec<u8>> {
    let slots = scan_exe_text_slots(data);
    let slots_by_offset = slots
        .iter()
        .map(|slot| (slot.offset, slot))
        .collect::<HashMap<_, _>>();
    let mut by_slot = BTreeMap::<usize, Vec<&TranslationEntry>>::new();
    for entry in entries {
        by_slot
            .entry(entry._slot_offset.context("EXE 条目缺少 _slot_offset")?)
            .or_default()
            .push(*entry);
    }
    let mut output = data.to_vec();
    for (slot_offset, slot_entries) in by_slot {
        let slot = slots_by_offset
            .get(&slot_offset)
            .with_context(|| format!("EXE 在 0x{slot_offset:X} 不再是可识别文本槽"))?;
        let mut patched = data[slot.offset..slot.offset + slot.capacity].to_vec();
        let mut edits = Vec::new();
        for entry in slot_entries {
            if entry._capacity != Some(slot.capacity)
                || entry._category.as_deref() != Some("exe_fixed_string")
            {
                bail!("EXE 条目 {} 的定长槽元数据已被修改", entry._index);
            }
            let span_index = entry._span.context("EXE 条目缺少 _span")?;
            let span = slot
                .spans
                .get(span_index)
                .with_context(|| format!("EXE 槽 0x{slot_offset:X} 不存在文本段 {span_index}"))?;
            validate_source_entry(entry, &span.text, &span.name)?;
            if is_changed(entry) {
                edits.push((span.start, span.end, encode_full_entry(entry, plan)?));
            }
        }
        if edits.is_empty() {
            continue;
        }
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
        for (start, end, replacement) in edits {
            patched.splice(start..end, replacement);
        }
        let encoded = serialize_expanded(game, &patched)?;
        if encoded.len() > slot.capacity {
            bail!(
                "EXE 定长槽 0x{slot_offset:X} 容量 {} 字节，译文编码后 {} 字节，超出 {} 字节",
                slot.capacity,
                encoded.len(),
                encoded.len() - slot.capacity
            );
        }
        output[slot.offset..slot.offset + slot.capacity].fill(0);
        output[slot.offset..slot.offset + encoded.len()].copy_from_slice(&encoded);
    }
    Ok(output)
}

fn encode_full_entry(entry: &TranslationEntry, plan: &EncodingPlan) -> Result<Vec<u8>> {
    if entry.message.contains(['\r', '\n']) {
        bail!(
            "条目 {} 的 message 含有直接换行；本系统文本框会自动换行",
            entry._index
        );
    }
    let mut output = Vec::new();
    if let Some(name) = &entry.name {
        if name.contains(['\r', '\n']) {
            bail!("条目 {} 的 name 含有换行", entry._index);
        }
        output.extend_from_slice(&[0x81, 0x79]);
        output.extend_from_slice(&encode_translation(name, plan)?);
        output.extend_from_slice(&[0x81, 0x7a]);
    }
    output.extend_from_slice(&encode_translation(&entry.message, plan)?);
    Ok(output)
}

fn validate_source_entry(
    entry: &TranslationEntry,
    source: &str,
    source_name: &Option<String>,
) -> Result<()> {
    if entry.scr_msg != source {
        bail!(
            "条目 {} 的 scr_msg 与原盘不符，请重新提取或恢复该字段",
            entry._index
        );
    }
    if &entry._scr_name != source_name {
        bail!(
            "条目 {} 的 _scr_name 与原盘不符，请重新提取或恢复该字段",
            entry._index
        );
    }
    Ok(())
}

fn is_changed(entry: &TranslationEntry) -> bool {
    entry.message != entry.scr_msg || entry.name != entry._scr_name
}

fn build_encoding_plan(game: GameId, entries: &[TranslationEntry]) -> Result<EncodingPlan> {
    if game == GameId::Nooch {
        let mut output_texts = Vec::new();
        let mut reserved_carriers = BTreeSet::new();
        for entry in entries {
            reserved_carriers.extend(strip_tokens(&entry.scr_msg).chars());
            if let Some(name) = &entry._scr_name {
                reserved_carriers.extend(strip_tokens(name).chars());
            }
            if is_changed(entry) {
                output_texts.push(strip_tokens(&entry.message));
                if let Some(name) = &entry.name {
                    output_texts.push(strip_tokens(name));
                }
            }
        }
        return EncodingPlan::build_remapped(
            output_texts.iter().map(String::as_str),
            reserved_carriers,
        )
        .map_err(anyhow::Error::msg);
    }

    let mut texts = Vec::new();
    for entry in entries {
        texts.push(strip_tokens(&entry.scr_msg));
        texts.push(strip_tokens(&entry.message));
        if let Some(name) = &entry._scr_name {
            texts.push(strip_tokens(name));
        }
        if let Some(name) = &entry.name {
            texts.push(strip_tokens(name));
        }
    }
    EncodingPlan::build(texts.iter().map(String::as_str)).map_err(anyhow::Error::msg)
}

fn changed_output_characters(entries: &[TranslationEntry]) -> Result<BTreeSet<char>> {
    let mut result = BTreeSet::new();
    for entry in entries.iter().filter(|entry| is_changed(entry)) {
        for text in std::iter::once(entry.message.as_str()).chain(entry.name.as_deref()) {
            for character in strip_tokens(text).chars() {
                let normalized = normalize_character(character).map_err(anyhow::Error::msg)?;
                if normalized != '　' {
                    result.insert(normalized);
                }
            }
        }
    }
    Ok(result)
}

fn load_game_map(game: GameId, disks: &[DiskSource]) -> Result<Vec<Vec<u8>>> {
    let name = match game {
        GameId::Nooch => "TEXT.MAP",
        GameId::Nooch2 => "SYSTEM.MAC",
        GameId::Nooch3 => "TEXTMAP.DAT",
    };
    let data = disks
        .iter()
        .flat_map(|disk| &disk.files)
        .find(|file| basename(&file.path).eq_ignore_ascii_case(name))
        .with_context(|| format!("{} 缺少 {name}", game.as_str()))?;
    match game {
        GameId::Nooch => load_nooch1_map(&data.data),
        GameId::Nooch2 => load_nooch2_map(&data.data),
        GameId::Nooch3 => load_nooch3_map(&data.data),
    }
}

struct SourceDisk<'a> {
    manifest: &'a DiskManifest,
    path: PathBuf,
    bytes: Vec<u8>,
    files: Vec<FdiFileData>,
}

fn load_source_disks<'a>(input: &Path, game: &'a GameManifest) -> Result<Vec<SourceDisk<'a>>> {
    let mut result = Vec::with_capacity(game.disks.len());
    for disk in &game.disks {
        let path = safe_join(input, &disk.source_relative)?;
        let bytes = fs::read(&path).with_context(|| format!("读取原盘 {} 失败", path.display()))?;
        let actual_hash = sha256_hex(&bytes);
        if actual_hash != disk.source_sha256 {
            bail!("原盘 {} 的 SHA-256 已变化，拒绝错盘注入", path.display());
        }
        let files =
            extract_fdi_files(&bytes, &path.to_string_lossy()).map_err(anyhow::Error::msg)?;
        result.push(SourceDisk {
            manifest: disk,
            path,
            bytes,
            files,
        });
    }
    Ok(result)
}

fn load_map_from_source_files(game: GameId, disks: &[SourceDisk<'_>]) -> Result<Vec<Vec<u8>>> {
    let name = match game {
        GameId::Nooch => "TEXT.MAP",
        GameId::Nooch2 => "SYSTEM.MAC",
        GameId::Nooch3 => "TEXTMAP.DAT",
    };
    let file = disks
        .iter()
        .flat_map(|disk| &disk.files)
        .find(|file| basename(&file.path).eq_ignore_ascii_case(name))
        .with_context(|| format!("原盘缺少 {name}"))?;
    match game {
        GameId::Nooch => load_nooch1_map(&file.data),
        GameId::Nooch2 => load_nooch2_map(&file.data),
        GameId::Nooch3 => load_nooch3_map(&file.data),
    }
}

fn validate_translation_document(entries: &[TranslationEntry], game: GameId) -> Result<()> {
    let mut keys = HashSet::new();
    let mut indices = HashSet::new();
    for entry in entries {
        if !matches!(entry._kind.as_str(), "message" | "exe_string") {
            bail!(
                "{} 条目 {} 的 _kind 必须是 message 或 exe_string",
                game.as_str(),
                entry._index
            );
        }
        if !keys.insert(entry.stable_key()) {
            bail!("{} JSON 中存在重复定位条目 {}", game.as_str(), entry._index);
        }
        if !indices.insert(entry._index) {
            bail!("{} JSON 中存在重复 _index {}", game.as_str(), entry._index);
        }
    }
    Ok(())
}

fn load_translation_entries(
    workspace: &Path,
    game: &GameManifest,
    game_id: GameId,
) -> Result<Vec<TranslationEntry>> {
    let mut result = Vec::new();
    if !game.translations.is_empty() {
        for document in &game.translations {
            let path = safe_join(workspace, &document.json)?;
            let entries: Vec<TranslationEntry> = read_json(&path)?;
            validate_translation_document(&entries, game_id)?;
            if entries.len() != document.entries {
                bail!(
                    "{} 的条目数 {} 与工作区清单 {} 不一致",
                    document.json,
                    entries.len(),
                    document.entries
                );
            }
            for entry in &entries {
                if entry._disk_sha256 != document.source_disk_sha256
                    || !entry._member.eq_ignore_ascii_case(&document.member)
                {
                    bail!("{} 内的源文件定位字段已被修改", document.json);
                }
            }
            result.extend(entries);
        }
    } else if let Some(messages) = &game.messages {
        let path = safe_join(workspace, messages)?;
        let entries: Vec<TranslationEntry> = read_json(&path)?;
        validate_translation_document(&entries, game_id)?;
        result = entries;
    } else {
        bail!("{} 工作区没有翻译 JSON 清单", game_id.as_str());
    }
    let mut locations = HashSet::new();
    for entry in &result {
        if !locations.insert(entry.stable_key()) {
            bail!("{} 的多个 JSON 含有重复源定位", game_id.as_str());
        }
    }
    Ok(result)
}

fn read_previous_translations(workspace: &Path) -> Result<BTreeMap<GameId, Vec<TranslationEntry>>> {
    let mut result = BTreeMap::new();
    let manifest_path = workspace.join("workspace.json");
    if !manifest_path.exists() {
        return Ok(result);
    }
    let manifest: WorkspaceManifest = read_json(&manifest_path)?;
    if manifest._format != WORKSPACE_FORMAT && manifest._format != LEGACY_WORKSPACE_FORMAT {
        return Ok(result);
    }
    for game in manifest.games {
        let game_id = game.game;
        result.insert(
            game_id,
            load_translation_entries(workspace, &game, game_id)?,
        );
    }
    Ok(result)
}

fn merge_existing_translations(fresh: &mut [TranslationEntry], old: &[TranslationEntry]) {
    let old_by_stable_key = old
        .iter()
        .map(|entry| (entry.stable_key(), entry))
        .collect::<HashMap<_, _>>();
    let old_by_source = old
        .iter()
        .map(|entry| (translation_source_key(entry), entry))
        .collect::<HashMap<_, _>>();
    for entry in fresh {
        if let Some(previous) = old_by_stable_key
            .get(&entry.stable_key())
            .copied()
            .or_else(|| old_by_source.get(&translation_source_key(entry)).copied())
        {
            if previous.scr_msg == entry.scr_msg && previous._scr_name == entry._scr_name {
                entry.message = previous.message.clone();
                entry.name = previous.name.clone();
            }
        }
    }
}

fn translation_source_key(entry: &TranslationEntry) -> String {
    format!(
        "{}|{}|{}|{}|{}|{:?}",
        entry._disk_sha256,
        entry._member.to_ascii_uppercase(),
        entry._kind,
        entry._ref,
        entry.scr_msg,
        entry._scr_name
    )
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut data = serde_json::to_vec_pretty(value)?;
    data.push(b'\n');
    fs::write(path, data).with_context(|| format!("写入 JSON {} 失败", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let data = fs::read(path).with_context(|| format!("读取 JSON {} 失败", path.display()))?;
    serde_json::from_slice(&data).with_context(|| format!("解析 JSON {} 失败", path.display()))
}

fn prepare_destination(path: &Path, overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        bail!("输出目录 {} 已存在；确认后使用 --overwrite", path.display());
    }
    if path.parent().is_none() {
        bail!("拒绝使用文件系统根目录作为输出");
    }
    Ok(())
}

fn temporary_sibling(path: &Path) -> Result<PathBuf> {
    let parent = path.parent().context("输出目录没有父目录")?;
    let name = path
        .file_name()
        .context("输出目录没有名称")?
        .to_string_lossy();
    Ok(parent.join(format!(".{name}.tmp-{}", std::process::id())))
}

fn clear_temporary(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)
            .with_context(|| format!("清理临时目录 {} 失败", path.display()))?;
    }
    Ok(())
}

fn commit_destination(temporary: &Path, destination: &Path, overwrite: bool) -> Result<()> {
    if destination.exists() {
        if !overwrite {
            bail!("输出目录 {} 已存在", destination.display());
        }
        fs::remove_dir_all(destination)
            .with_context(|| format!("删除旧输出 {} 失败", destination.display()))?;
    }
    fs::rename(temporary, destination).with_context(|| {
        format!(
            "提交临时目录 {} 到 {} 失败",
            temporary.display(),
            destination.display()
        )
    })
}

fn same_path(left: &Path, right: &Path) -> Result<bool> {
    let normalize = |path: &Path| -> Result<PathBuf> {
        if path.exists() {
            Ok(fs::canonicalize(path)?)
        } else if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            Ok(std::env::current_dir()?.join(path))
        }
    };
    Ok(normalize(left)? == normalize(right)?)
}

fn main_exe_name(game: GameId) -> &'static str {
    match game {
        GameId::Nooch => "NOOCH.EXE",
        GameId::Nooch2 => "NOOCH2.EXE",
        GameId::Nooch3 => "NK3.EXE",
    }
}
