use crate::{
    codec,
    codec::Record,
    dictionary, json, load_game,
    output::{read_tree, Job},
    patch_file, read, Game, Primary, Result,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
use vn_font::font_98::{self, EncodingPlan, SubstitutionMap};

const PRIMARY: &str = ".uma-primary.json";
const EHNAME_ACTIVE: usize = 1838;
const EHNAME_SLOTS: usize = 8700;
const EHNAME_PAIRS: usize = 0x43FA;
const MAX_COMPOSED_NAME_BYTES: usize = 18;

pub(crate) fn source_inputs(source: &Path) -> Vec<PathBuf> {
    crate::DISKS.iter().map(|name| source.join(name)).collect()
}

pub fn prepare_export(source: &Path, output: &Path, overwrite: bool) -> Result<Job> {
    let game = load_game(source)?;
    let mut files = BTreeMap::new();
    let mut messages = 0;
    for unit in &game.primary.units {
        let entries = unit.entries();
        messages += entries.len();
        files.insert(
            PathBuf::from("translations").join(unit.translation_path()),
            vn_text::write_template(&entries).map_err(|e| e.to_string())?,
        );
    }
    files.insert(PathBuf::from(PRIMARY), json(&game.primary)?);
    files.insert(PathBuf::from("说明.txt"), "只编辑 translations 中 JSON 的 message；保持条目顺序、格式占位符及原生字形标记。\nMES 的 86 AB 表情已从翻译文本滤掉，导入时保留在原行尾。\nMES 每条最多 112 字节；EXE 译文不能超出原始字符串槽位。\nEHPARTS.NPA/NPB 每项是 XOR FF 的 14 字节词典槽；新译文由本次字库计划编码后限单行 14 字节，不接受控制符、标记或格式指令；id 0 保持空。\nJNAME.DAT 每项是 XOR FF 的 8 字节骑手名槽，新译文会去掉末尾空白并在字库计划编码后检查容量。\nRNAME.DAT 每项是 XOR FF 的 12 字节赛事名槽。NPA/NPB 改动时，导入会通过 EHNAME.TMP 检查拼接后的活动马名不超过 18 个实际编码字节。\n.uma-primary.json 为来源与定位记录，请勿编辑。\n允许把部分 JSON 按相同子目录结构放入独立译文目录；缺少的文件沿用原文。\n".as_bytes().to_vec());
    let report = vn_cli::RunReport {
        summary: "已准备 MES、游戏 EXE 与马名词典翻译工作区".into(),
        totals: vec![
            ("文本文件".into(), game.primary.units.len() as u64),
            ("文本条目".into(), messages as u64),
        ],
        ..Default::default()
    };
    Job::new(source_inputs(source), output, overwrite, files, report)
}

pub fn prepare_memory_fix(
    source: &Path,
    hdi_base: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<Job> {
    let game = load_game(source)?;
    let himem = game
        .files
        .get("01_DISK/HIMEM.SYS")
        .ok_or("原盘缺少 01_DISK/HIMEM.SYS")?;
    let base = read(hdi_base)?;
    let image =
        crate::boot_memory::add_boot_files(&base, himem, crate::boot_memory::ConfigPolicy::Ensure)?;
    let report = vn_cli::RunReport {
        summary: "已准备仅补充 DOS 高端内存启动文件的新 HDI".into(),
        totals: vec![("新建 HDI".into(), 1)],
        ..Default::default()
    };
    let mut files = BTreeMap::new();
    files.insert(PathBuf::from("game.hdi"), image);
    files.insert(PathBuf::from("boot-memory.json"), json(&serde_json::json!({
        "version": 1, "boot_files": ["HIMEM.SYS", "CONFIG.SYS"],
        "image_changed": base != files[&PathBuf::from("game.hdi")],
        "config": String::from_utf8_lossy(crate::boot_memory::CONFIG),
        "font_update_required": false,
        "preservation": "Only newly allocated FAT12 clusters, matching FAT12 entries and root directory entries may change; all existing files and DOS data remain byte-identical."
    }))?);
    files.insert(PathBuf::from("使用说明.txt"), "新 HDI 仅补充 HIMEM.SYS 与 CONFIG.SYS（DOS=HIGH）；不会重装游戏或改动存档。原有 NP2 外置字库无需更新。\n".as_bytes().to_vec());
    let mut inputs = source_inputs(source);
    inputs.push(hdi_base.to_path_buf());
    Job::new(inputs, output, overwrite, files, report)
}

fn reserve_pair(pair: [u8; 2], reserved: &mut BTreeSet<u16>) {
    if let Ok(jis) = font_98::cp932_to_jis(pair) {
        // NP2 loads these pages; game-owned JIS 7621+ bitmaps remain in FONT.TXT.
        let page = jis[0].saturating_sub(0x20);
        if (1..=0x55).contains(&page) || (0x58..=0x5f).contains(&page) {
            reserved.insert(u16::from_be_bytes(pair));
        }
    }
}
fn reserve_scan(bytes: &[u8], reserved: &mut BTreeSet<u16>) {
    for pair in bytes.windows(2) {
        reserve_pair([pair[0], pair[1]], reserved);
    }
}

fn plan_font(
    game: &Game,
    translated: &BTreeMap<String, Vec<vn_text::Entry>>,
) -> Result<(EncodingPlan, BTreeSet<u16>)> {
    let mut reserved = BTreeSet::new();
    let mut final_texts = Vec::new();
    for unit in &game.primary.units {
        let entries = translated.get(&unit.path).ok_or("缺少已合并文本")?;
        for (index, (record, entry)) in unit.records.iter().zip(entries).enumerate() {
            if unit.xor_width.is_some() {
                if entry.message != record.message {
                    dictionary::validate_translation(&entry.message)
                        .map_err(|e| format!("{} 第 {index} 条: {e}", unit.path))?;
                }
            } else {
                codec::validate_translation(record, &entry.message, unit.mes)
                    .map_err(|e| format!("{} 第 {index} 条: {e}", unit.path))?;
            }
            final_texts.push(codec::font_text(&entry.message)?);
            if entry.message == record.message {
                let decoded = if unit.mes || unit.xor_width.is_some() {
                    record.raw.iter().map(|b| b ^ 0xff).collect()
                } else {
                    record.raw.clone()
                };
                reserve_scan(&decoded, &mut reserved);
            }
            if !record.expression_lines.is_empty() {
                reserve_pair([0x86, 0xab], &mut reserved);
            }
            let mut pos = 0;
            while pos < record.message.len() {
                let rest = &record.message[pos..];
                if let Some((len, bytes)) = codec::marker(rest)? {
                    if bytes.len() == 2 {
                        reserve_pair([bytes[0], bytes[1]], &mut reserved);
                    }
                    pos += len;
                } else {
                    pos += rest.chars().next().ok_or("文本截断")?.len_utf8();
                }
            }
        }
    }
    for (path, bytes) in &game.files {
        // These unchanged game data tables contain names/weights, not image data.
        // Both stored and NOT-decoded views conservatively protect their native glyphs.
        if path.ends_with(".TMP") || path.ends_with("/JNAME.DAT") || path.ends_with("/RNAME.DAT") {
            reserve_scan(bytes, &mut reserved);
            let decoded: Vec<_> = bytes.iter().map(|b| b ^ 0xff).collect();
            reserve_scan(&decoded, &mut reserved);
        }
    }
    let executable = game.files.get("02_DISK/UMANARI.EXE").ok_or("缺少主程序")?;
    reserve_scan(
        executable
            .get(0x5b524..0x5b59c)
            .ok_or("主程序字符键盘表越界")?,
        &mut reserved,
    );
    let substitutions = SubstitutionMap::embedded()?;
    let plan = EncodingPlan::build(
        &substitutions,
        reserved.iter().copied(),
        final_texts.iter().map(String::as_str),
    )?;
    Ok((plan, reserved))
}

fn validate_name_pairs(
    table: &[u8],
    source_npb: &[Record],
    translated_npb: &[vn_text::Entry],
    source_npa: &[Record],
    translated_npa: &[vn_text::Entry],
    plan: &EncodingPlan,
) -> Result<()> {
    let table_end = EHNAME_PAIRS
        .checked_add(EHNAME_SLOTS * 4)
        .ok_or("EHNAME 名称索引偏移溢出")?;
    if table.len() < table_end {
        return Err(format!(
            "EHNAME.TMP 长度不足：至少需要 {table_end} 字节，实际 {}",
            table.len()
        ));
    }
    let active = table.get(..2).ok_or("EHNAME.TMP 缺少活动名称计数")?;
    let active = usize::from(u16::from_le_bytes([active[0], active[1]]));
    if active != EHNAME_ACTIVE {
        return Err(format!(
            "EHNAME.TMP 活动名称数应为 {EHNAME_ACTIVE}，实际 {active}"
        ));
    }
    if translated_npb.len() != source_npb.len() || translated_npa.len() != source_npa.len() {
        return Err("马名词段译文条数与原字典不一致".into());
    }
    for id in 0..EHNAME_SLOTS {
        let offset = EHNAME_PAIRS + id * 4;
        let npb_id = usize::from(u16::from_le_bytes(
            table[offset..offset + 2]
                .try_into()
                .map_err(|_| "EHNAME.TMP NPB 引用被截断")?,
        ));
        let npa_id = usize::from(u16::from_le_bytes(
            table[offset + 2..offset + 4]
                .try_into()
                .map_err(|_| "EHNAME.TMP NPA 引用被截断")?,
        ));
        if npb_id >= source_npb.len() {
            return Err(format!(
                "EHNAME 名称 id {id} 引用越界的 NPB 词段 id {npb_id}"
            ));
        }
        if npa_id >= source_npa.len() {
            return Err(format!(
                "EHNAME 名称 id {id} 引用越界的 NPA 词段 id {npa_id}"
            ));
        }
        if id >= active {
            continue;
        }
        let old_name = format!(
            "{}{}",
            source_npb[npb_id].message, source_npa[npa_id].message
        );
        let new_name = format!(
            "{}{}",
            translated_npb[npb_id].message, translated_npa[npa_id].message
        );
        if new_name == old_name {
            continue;
        }
        let length = codec::encode(&new_name, plan)?.len();
        if length > MAX_COMPOSED_NAME_BYTES {
            return Err(format!(
                "EHNAME 名称 id {id} 拼接后超长：{old_name:?} → {new_name:?}，{length} 字节，最多 {MAX_COMPOSED_NAME_BYTES} 字节"
            ));
        }
    }
    Ok(())
}

fn validate_changed_name_pairs(
    game: &Game,
    translated: &BTreeMap<String, Vec<vn_text::Entry>>,
    plan: &EncodingPlan,
) -> Result<()> {
    let unit = |path: &str| {
        game.primary
            .units
            .iter()
            .find(|unit| unit.path == path)
            .ok_or_else(|| format!("缺少名称词典单元 {path}"))
    };
    let npb = unit("03_DISK/EHPARTS.NPB")?;
    let npa = unit("03_DISK/EHPARTS.NPA")?;
    let entries_npb = translated.get(&npb.path).ok_or("缺少 NPB 译文")?;
    let entries_npa = translated.get(&npa.path).ok_or("缺少 NPA 译文")?;
    if npb
        .records
        .iter()
        .zip(entries_npb)
        .all(|(record, entry)| record.message == entry.message)
        && npa
            .records
            .iter()
            .zip(entries_npa)
            .all(|(record, entry)| record.message == entry.message)
    {
        return Ok(());
    }
    let table = game
        .files
        .get("02_DISK/EHNAME.TMP")
        .ok_or("缺少 EHNAME.TMP")?;
    validate_name_pairs(
        table,
        &npb.records,
        entries_npb,
        &npa.records,
        entries_npa,
        plan,
    )
}

pub struct ImportOptions<'a> {
    pub source: &'a Path,
    pub workspace: &'a Path,
    pub translations: Option<&'a Path>,
    pub font: Option<&'a Path>,
    pub hdi_base: Option<&'a Path>,
    pub face: &'a str,
    pub output: &'a Path,
    pub overwrite: bool,
}

pub fn prepare_import(options: ImportOptions<'_>) -> Result<Job> {
    let game = load_game(options.source)?;
    let stored: Primary = serde_json::from_slice(&read(&options.workspace.join(PRIMARY))?)
        .map_err(|e| format!("来源记录损坏: {e}"))?;
    // Full native records and source ranges bind the template without hashes.
    if stored != game.primary {
        return Err("当前原盘、完整原文或来源范围与导出工作区不同；请使用对应原盘".into());
    }
    let root = options
        .translations
        .map(Path::to_path_buf)
        .unwrap_or_else(|| options.workspace.join("translations"));
    let root = if root.join("translations").is_dir() {
        root.join("translations")
    } else {
        root
    };
    let supplied_files = read_tree(&root)?;
    let mut supplied = BTreeMap::new();
    for (path, bytes) in supplied_files {
        let unit = game
            .primary
            .units
            .iter()
            .find(|u| u.translation_path() == path)
            .ok_or_else(|| format!("译文目录含未知文件: {}", path.display()))?;
        let mut entries =
            vn_text::read_template(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        if unit.xor_width == Some(8) {
            for entry in &mut entries {
                entry.message = entry.message.trim_end().to_owned();
            }
        }
        vn_text::validate_shape(&unit.entries(), &entries)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        supplied.insert(unit.path.clone(), entries);
    }
    for unit in &game.primary.units {
        supplied
            .entry(unit.path.clone())
            .or_insert_with(|| unit.entries());
    }
    let (plan, reserved) = plan_font(&game, &supplied)?;
    validate_changed_name_pairs(&game, &supplied, &plan)?;
    let mut rebuilt = game.files.clone();
    let (mut changed_messages, mut changed_files) = (0usize, 0usize);
    for unit in &game.primary.units {
        let data = rebuilt.get_mut(&unit.path).ok_or("缺少原始文件")?;
        let entries = &supplied[&unit.path];
        let mut changed = false;
        for (index, (record, entry)) in unit.records.iter().zip(entries).enumerate() {
            let encoded = if let Some(width) = unit.xor_width {
                dictionary::rebuild(record, &entry.message, &plan, width)
            } else {
                codec::rebuild(record, &entry.message, unit.mes, &plan)
            }
            .map_err(|e| format!("{} 第 {index} 条 (0x{:X}): {e}", unit.path, record.offset))?;
            if entry.message != record.message {
                changed_messages += 1;
                changed = true;
            }
            let end = record
                .offset
                .checked_add(encoded.len())
                .ok_or("文本范围溢出")?;
            data.get_mut(record.offset..end)
                .ok_or("文本范围超出文件")?
                .copy_from_slice(&encoded);
        }
        changed_files += usize::from(changed);
    }
    let font_source = options.font.map(read).transpose()?;
    let font = font_98::prepare_font(
        font_source.as_deref().unwrap_or(font_98::EMBEDDED_FONT),
        &plan.requests(),
        &reserved,
        options.face,
    )?;
    let mut products = BTreeMap::new();
    for disk in &game.disks {
        let mut image = disk.bytes.clone();
        for identity in &disk.files {
            let data = rebuilt.get(&identity.path).ok_or("重建文件集合不完整")?;
            patch_file(&mut image, identity, data)?;
            products.insert(PathBuf::from("files").join(&identity.path), data.clone());
        }
        products.insert(PathBuf::from("images").join(&disk.name), image);
    }
    let installed = crate::install::build(&rebuilt)?;
    let installed_count = installed.len();
    let start_batch = b"@ECHO OFF\r\nCD \\MV\\UMA\r\nUMA.EXE\r\n";
    if let Some(path) = options.hdi_base {
        let base = read(path)?;
        let himem = game
            .files
            .get("01_DISK/HIMEM.SYS")
            .ok_or("原盘缺少 HIMEM.SYS")?;
        let hdi = crate::hdi::patch_installed(&base, &installed, start_batch, himem)?;
        products.insert(PathBuf::from("game.hdi"), hdi);
    }
    products.extend(installed);
    products.insert(PathBuf::from("installed/START.BAT"), start_batch.to_vec());
    products.insert(PathBuf::from("font.tmp"), font.bytes);
    products.insert(
        PathBuf::from("font-plan.json"),
        json(&plan.manifest_entries()?)?,
    );
    products.insert(PathBuf::from("reserved-cp932.json"), json(&reserved)?);
    let mut build = serde_json::json!({
        "schema": "uma-pc98-build-v1", "changed_messages": changed_messages,
        "changed_files": changed_files, "files": rebuilt.len(), "images": game.disks.len(),
        "patched_glyphs": font.patched_glyphs, "font_face": options.face,
        "installed_files": installed_count, "installed_directory": "installed/MV/UMA",
        "boot_media": "installation floppies; install to a DOS hard disk before running UMA.EXE",
        "font_source": options.font.and_then(Path::file_name).map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "vn-font embedded PC98 font.tmp".into()),
        "layout": "fixed MES/EXE slots, XOR-FF 8/12/14-byte name tables, and original FAT12 cluster chains", "hash_checks": false,
        "expression_86ab": "filtered from translation; preserved at native line ends"
    });
    let mut usage = "这四张 D88 是原版安装盘，不能当作已安装的硬盘直接进入游戏。\n第一张盘的 AUTOEXEC.BAT 会寻找 B:\\MV\\UMA\\UMA.EXE，未安装时显示“ゲームが見つかりません”。\n方法一：挂载可写的 DOS HDI，从第一张盘启动后输入 INST，按提示轮换四张盘并安装到硬盘。\n方法二：将 installed 中的 MV 文件夹导入已有 DOS HDI 的根目录；启动 DOS 后先切换到该硬盘盘符，再执行 CD \\MV\\UMA，然后执行 UMA。\ninstalled/START.BAT 可在已切换到目标硬盘盘符后运行。\ninstalled 按原版 FILELIST.INF 合并四盘，已把 UMA0.EXE 改名为 UMA.EXE，并生成各存档槽的初始文件。\nimages 为四张重建安装盘，files 为完整盘内文件集合。\n将本目录 font.tmp 放到 NP2 模拟器字体目录使用，字库与本次镜像或硬盘游戏目录配套。\n原盘、原文件和译文均未覆盖；86 AB 表情及 FONT.TXT 自定义字形保持原生形式。\n".to_owned();
    if options.hdi_base.is_some() {
        let build = build.as_object_mut().ok_or("构建记录不是 JSON 对象")?;
        build.insert("hdi_output".into(), serde_json::json!("game.hdi"));
        build.insert("boot_media".into(), serde_json::json!("D88 are installation floppies; game.hdi inherits its base DOS boot state and does not auto-start UMA.EXE"));
        build.insert(
            "hdi_other_dos_files_preserved".into(),
            serde_json::json!(true),
        );
        build.insert("hdi_preservation".into(), serde_json::json!("Existing files remain intact except matched game files and optional equal-length root START.BAT; when boot files are missing, only their newly allocated FAT12 clusters, FAT entries and root directory entries also change. Existing CONFIG.SYS is preserved."));
        build.insert("hdi_boot_memory".into(), serde_json::json!("adds original HIMEM.SYS and CONFIG.SYS with DOS=HIGH only when CONFIG.SYS is absent; an existing CONFIG.SYS and its driver selection remain unchanged"));
        build.insert(
            "hdi_save_data".into(),
            serde_json::json!("fresh initial save slots; no save migration"),
        );
        usage.push_str("如本次输出 game.hdi，它沿用基础 HDI 的 DOS 启动状态，不自动启动游戏；缺少时会添加 HIMEM.SYS 与 DOS=HIGH 配置，已有 CONFIG.SYS 原样保留。进入 DOS 后执行 CD \\MV\\UMA、UMA。HDI 中的初始存档来自本次安装文件，不迁移旧进度。\n");
    }
    products.insert(PathBuf::from("build.json"), json(&build)?);
    products.insert(PathBuf::from("使用说明.txt"), usage.into_bytes());
    let mut inputs = source_inputs(options.source);
    inputs.extend([options.workspace.join(PRIMARY), root]);
    if let Some(path) = options.font {
        inputs.push(path.to_path_buf());
    }
    if let Some(path) = options.hdi_base {
        inputs.push(path.to_path_buf());
    }
    let mut totals = vec![
        ("修改条目".into(), changed_messages as u64),
        ("修改文件".into(), changed_files as u64),
        ("文件数".into(), rebuilt.len() as u64),
        ("磁盘数".into(), game.disks.len() as u64),
        ("重绘字形".into(), font.patched_glyphs as u64),
        ("安装后文件数".into(), installed_count as u64),
    ];
    if options.hdi_base.is_some() {
        totals.push(("新建 HDI".into(), 1));
    }
    let report = vn_cli::RunReport {
        summary: if options.hdi_base.is_some() {
            "已准备文本注入、PC98 字库、四张 D88 和新 HDI".into()
        } else {
            "已准备文本注入、PC98 字库和四张 D88 重封".into()
        },
        totals,
        ..Default::default()
    };
    Job::new(inputs, options.output, options.overwrite, products, report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vn_font::font_98::{EncodingPlan, SubstitutionMap};

    fn record(message: &str, offset: usize) -> Record {
        Record {
            offset,
            raw: Vec::new(),
            message: message.into(),
            expression_lines: Vec::new(),
        }
    }

    fn entries(messages: &[&str]) -> Vec<vn_text::Entry> {
        messages
            .iter()
            .map(|message| vn_text::Entry {
                message: (*message).into(),
                name: None,
            })
            .collect()
    }

    fn plan(entries: &[vn_text::Entry]) -> EncodingPlan {
        let display = entries
            .iter()
            .map(|entry| codec::font_text(&entry.message).unwrap())
            .collect::<Vec<_>>();
        EncodingPlan::build(
            &SubstitutionMap::embedded().unwrap(),
            [],
            display.iter().map(String::as_str),
        )
        .unwrap()
    }

    fn ehname(pair: (u16, u16)) -> Vec<u8> {
        let end = EHNAME_PAIRS + EHNAME_SLOTS * 4;
        let mut table = vec![0; end];
        table[..2].copy_from_slice(&(EHNAME_ACTIVE as u16).to_le_bytes());
        table[EHNAME_PAIRS..EHNAME_PAIRS + 2].copy_from_slice(&pair.0.to_le_bytes());
        table[EHNAME_PAIRS + 2..EHNAME_PAIRS + 4].copy_from_slice(&pair.1.to_le_bytes());
        table
    }

    #[test]
    fn translated_wordpieces_cannot_make_composed_names_exceed_eighteen_bytes() {
        let source_npb = vec![record("", 0), record("原", 1)];
        let source_npa = vec![record("", 0), record("名", 1)];
        let segment = "蓝调蓝调蓝";
        let target_npb = entries(&["", segment]);
        let target_npa = entries(&["", segment]);
        let final_entries = [target_npb.clone(), target_npa.clone()].concat();
        let plan = plan(&final_entries);
        let error = validate_name_pairs(
            &ehname((1, 1)),
            &source_npb,
            &target_npb,
            &source_npa,
            &target_npa,
            &plan,
        )
        .unwrap_err();
        assert!(error.contains("名称 id 0"));
        assert!(error.contains("20 字节"));
    }

    #[test]
    fn unchanged_long_names_are_allowed_and_pair_references_are_bounded() {
        let long = "ABCDEFGHIJKLMNOPQRST";
        let source_npb = vec![record("", 0), record(long, 1)];
        let source_npa = vec![record("", 0), record("", 1)];
        let target_npb = entries(&["", long]);
        let target_npa = entries(&["", ""]);
        let final_entries = [target_npb.clone(), target_npa.clone()].concat();
        let plan = plan(&final_entries);
        let mut table = ehname((1, 0));
        assert!(validate_name_pairs(
            &table,
            &source_npb,
            &target_npb,
            &source_npa,
            &target_npa,
            &plan,
        )
        .is_ok());
        table[EHNAME_PAIRS..EHNAME_PAIRS + 2].copy_from_slice(&2u16.to_le_bytes());
        assert!(validate_name_pairs(
            &table,
            &source_npb,
            &target_npb,
            &source_npa,
            &target_npa,
            &plan,
        )
        .unwrap_err()
        .contains("NPB 词段 id 2"));
    }
}
