use crate::{
    fds::{Disk, File, Side},
    hex, json_bytes, sha256,
    text::{terminated, Codec},
    Result,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const PROGRAM_SHA: &str = "5758ef43b0694cb8c592c473036b1c0cd235bdd0502d466f2de609aa35793f38";
// FDS vectors at DFF6..DFFF must survive every DATA load.
pub const DATA_LIMIT: usize = 0xDFF6 - 0xA200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub index: usize,
    pub cpu_address: u16,
    pub file_offset: usize,
    pub pointer_file_offset: usize,
    pub raw_hex: String,
    pub source_bytes: usize,
    pub original: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<crate::ui::Locator>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub cpu_address: u16,
    pub action: u8,
    pub coordinates: [u8; 4],
    pub conditions: Vec<[u8; 2]>,
    pub writes: Vec<[u8; 2]>,
    pub raw_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub id: u8,
    pub event_pointer: u16,
    pub events: Vec<Event>,
    pub initial_pointer: u16,
    pub initial_states: Vec<u8>,
    pub conditional_writes: Vec<[u8; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub name: String,
    pub side: usize,
    pub file_index: usize,
    pub header_words: [u16; 7],
    pub texts: Vec<Text>,
    pub scenes: Vec<Scene>,
}

impl Resource {
    pub fn view(&self) -> Vec<vn_text::Entry> {
        self.texts
            .iter()
            .map(|t| vn_text::Entry {
                name: None,
                message: t.original.clone(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Native {
    pub schema: u32,
    pub source_sha256: String,
    pub header_size: usize,
    pub files: Vec<File>,
    pub sides: Vec<Side>,
    pub dictionary_hex: Vec<String>,
    pub resources: Vec<Resource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub schema: u32,
    pub source_sha256: String,
    pub native_sha256: String,
    pub templates: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub disk: Disk,
    pub codec: Codec,
    pub native: Native,
}

#[derive(Debug)]
pub struct Rebuild {
    pub image: Vec<u8>,
    pub changed_entries: usize,
    pub grown_files: Vec<(String, usize, usize)>,
}

impl Game {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let disk = Disk::parse(bytes)?;
        if disk.sides.len() != 2 || disk.files.len() != 26 {
            return Err("当前适配器要求已研究的双面、26 文件水晶之龙布局".into());
        }
        for s in &disk.sides {
            let base = disk.header_size + s.index * crate::fds::SIDE_SIZE;
            if &bytes[base + 16..base + 19] != b"SSD" {
                return Err("磁盘游戏标识不是 SSD（水晶之龙）".into());
            }
        }
        let program = disk.file("ADVS")?;
        if program.load_address != 0x6000 || program.kind != 0 || program.sha256 != PROGRAM_SHA {
            return Err("ADVS 与已确认的 6502 引擎不符，拒绝使用其文本／字库地址".into());
        }
        let macros = disk.file("MSGMACRO")?;
        if macros.kind != 0 || macros.load_address != 0x400 || macros.size != 768 {
            return Err("MSGMACRO 装载布局不符".into());
        }
        let codec = Codec::parse_macros(disk.payload(macros))?;
        let fixed = disk.file("FIXDAT")?;
        if fixed.kind != 0 || fixed.load_address != 0x9000 || fixed.size != 0x780 {
            return Err("FIXDAT 装载布局不符".into());
        }
        let mut resources = Vec::new();
        for index in 0..10 {
            let file = disk.file(&format!("DATA{index}"))?;
            if file.kind != 0 || file.load_address != 0xA200 || file.size > DATA_LIMIT {
                return Err(format!("{} 装载范围不符／覆盖 FDS 向量", file.name).into());
            }
            resources.push(parse_resource(file, disk.payload(file), &codec)?);
        }
        resources.extend(crate::ui::parse(fixed, disk.payload(fixed), &codec)?);
        resources.push(crate::ui::parse_engine(
            program,
            disk.payload(program),
            &codec,
        )?);
        let mut ids = resources
            .iter()
            .flat_map(|r| r.scenes.iter().map(|s| s.id))
            .collect::<Vec<_>>();
        ids.sort();
        if ids != (0..51).collect::<Vec<_>>() {
            return Err("场景目录未完整覆盖 0–50，或有重复场景".into());
        }
        let native = Native {
            schema: 2,
            source_sha256: sha256(bytes),
            header_size: disk.header_size,
            files: disk.files.clone(),
            sides: disk.sides.clone(),
            dictionary_hex: codec.macros.iter().map(|m| hex(m)).collect(),
            resources,
        };
        Ok(Self {
            disk,
            codec,
            native,
        })
    }

    pub fn binding(&self) -> Result<Binding> {
        Ok(Binding {
            schema: 2,
            source_sha256: self.native.source_sha256.clone(),
            native_sha256: sha256(&json_bytes(&self.native)?),
            templates: self
                .native
                .resources
                .iter()
                .map(|r| {
                    Ok((
                        format!("{}.json", r.name),
                        sha256(&vn_text::write_template(&r.view())?),
                    ))
                })
                .collect::<Result<_>>()?,
        })
    }

    pub fn export_files(&self) -> Result<BTreeMap<String, Vec<u8>>> {
        let mut files = BTreeMap::from([
            ("source.fds".into(), self.disk.bytes().to_vec()),
            ("native.json".into(), json_bytes(&self.native)?),
            (
                "translations/binding.json".into(),
                json_bytes(&self.binding()?)?,
            ),
        ]);
        for r in &self.native.resources {
            files.insert(
                format!("translations/{}.json", r.name),
                vn_text::write_template(&r.view())?,
            );
        }
        for f in &self.disk.files {
            files.insert(
                format!("disk_files/{}", f.output_name()),
                self.disk.payload(f).to_vec(),
            );
        }
        files.extend(crate::font::export(self)?);
        Ok(files)
    }

    pub fn import(&self, translations: &BTreeMap<String, Vec<vn_text::Entry>>) -> Result<Rebuild> {
        for name in translations.keys() {
            if !self.native.resources.iter().any(|r| r.name == *name) {
                return Err(format!("未知翻译源单元: {name}").into());
            }
        }
        let mut replacements = BTreeMap::new();
        let mut changed = 0;
        let mut grown = Vec::new();
        for r in &self.native.resources {
            let Some(supplied) = translations.get(&r.name) else {
                continue;
            };
            let expected = r.view();
            vn_text::validate_shape(&expected, supplied)?;
            let count = expected
                .iter()
                .zip(supplied)
                .filter(|(a, b)| a.message != b.message)
                .count();
            if count != 0 && !r.name.starts_with("DATA") {
                return Err(
                    "道具和系统中文回注需要 convert 后的 MMC5 NES；FDS 原布局不重排这些界面表"
                        .into(),
                );
            }
            if count == 0 {
                continue;
            }
            let file = self.disk.file(&r.name)?;
            let original = self.disk.payload(file);
            let mut streams = Vec::new();
            for (text, entry) in r.texts.iter().zip(supplied) {
                let bytes = if text.original == entry.message {
                    terminated(original, text.file_offset)?.to_vec()
                } else {
                    self.codec
                        .encode(&entry.message)
                        .map_err(|e| format!("{}[{}]: {e}", r.name, text.index))?
                };
                streams.push(bytes);
            }
            let payload = relocate_texts(original, r, &streams)?;
            if payload.len() > original.len() {
                grown.push((r.name.clone(), original.len(), payload.len()));
            }
            changed += count;
            replacements.insert(r.name.clone(), payload);
        }
        let image = self.disk.rebuild(&replacements)?;
        let reparsed = Self::parse(&image)?;
        for (old, new) in self.native.resources.iter().zip(&reparsed.native.resources) {
            let expected = translations
                .get(&old.name)
                .cloned()
                .unwrap_or_else(|| old.view());
            if expected != new.view() {
                return Err(format!("{} 的重提取结果与合入译文不符", old.name).into());
            }
            if old.scenes != new.scenes {
                return Err("回注改变了场景事件／初始化数据".into());
            }
        }
        Ok(Rebuild {
            image,
            changed_entries: changed,
            grown_files: grown,
        })
    }

    pub fn summary(&self) -> String {
        format!(
            "2 面 / {} 文件 / {} 主表文本 / {} 场景 / {} 条事件",
            self.disk.files.len(),
            self.native
                .resources
                .iter()
                .filter(|r| r.name.starts_with("DATA"))
                .map(|r| r.texts.len())
                .sum::<usize>(),
            self.native
                .resources
                .iter()
                .map(|r| r.scenes.len())
                .sum::<usize>(),
            self.native
                .resources
                .iter()
                .flat_map(|r| &r.scenes)
                .map(|s| s.events.len())
                .sum::<usize>()
        )
    }
}

fn offset(raw: &[u8], cpu: u16, size: usize) -> Result<usize> {
    let at = usize::from(cpu)
        .checked_sub(0xA200)
        .ok_or("DATA 指针小于 $A200")?;
    if at + size > raw.len() {
        return Err(format!("DATA 指针 ${cpu:04X}／范围 {size} 越界").into());
    }
    Ok(at)
}

fn word(raw: &[u8], at: usize) -> Result<u16> {
    let bytes = raw.get(at..at + 2).ok_or("地址字截断")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn parse_resource(file: &File, raw: &[u8], codec: &Codec) -> Result<Resource> {
    parse_resource_with(file, raw, |at| {
        let source = terminated(raw, at)?;
        Ok((source.len(), codec.decode(source)?))
    })
}

pub(crate) fn parse_resource_with(
    file: &File,
    raw: &[u8],
    resolve: impl Fn(usize) -> Result<(usize, String)>,
) -> Result<Resource> {
    let mut header_words = [0; 7];
    for (i, value) in header_words.iter_mut().enumerate() {
        *value = word(raw, 2 * i)?;
        offset(raw, *value, 1)?;
    }
    let table = offset(raw, header_words[2], 1)?;
    let count = usize::from(raw[table]);
    offset(raw, header_words[2], 1 + 2 * count)?;
    let mut texts = Vec::new();
    for index in 0..count {
        let pointer_at = table + 1 + 2 * index;
        let cpu = word(raw, pointer_at)?;
        let at = offset(raw, cpu, 1)?;
        let (length, message) = resolve(at)?;
        let source = raw.get(at..at + length).ok_or("文本范围越界")?;
        texts.push(Text {
            index,
            cpu_address: cpu,
            file_offset: at,
            pointer_file_offset: pointer_at,
            raw_hex: hex(source),
            source_bytes: source.len(),
            original: message,
            ui: None,
            ui_reference: None,
        });
    }
    let initial = offset(raw, header_words[6], 1)?;
    let initial_count = usize::from(raw[initial]);
    offset(raw, header_words[6], 1 + 2 * initial_count)?;
    let mut scenes = Vec::new();
    let mut at = 14;
    loop {
        let id = *raw.get(at).ok_or("场景目录缺少 FF")?;
        if id == 255 {
            break;
        }
        let pointer = word(raw, at + 1)?;
        let mut pos = offset(raw, pointer, 1)?;
        let mut events = Vec::new();
        while *raw.get(pos).ok_or("事件列表缺少 FF")? != 255 {
            let head = raw.get(pos..pos + 7).ok_or("事件头截断")?;
            let n = usize::from(head[5]);
            let m = usize::from(head[6]);
            let event = raw.get(pos..pos + 7 + 2 * (n + m)).ok_or("事件记录截断")?;
            let conditions = event[7..7 + 2 * n]
                .chunks_exact(2)
                .map(|p| [p[0], p[1]])
                .collect();
            let writes = event[7 + 2 * n..]
                .chunks_exact(2)
                .map(|p| [p[0], p[1]])
                .collect();
            events.push(Event {
                cpu_address: u16::try_from(pos + 0xA200)?,
                action: head[0],
                coordinates: [head[1], head[2], head[3], head[4]],
                conditions,
                writes,
                raw_hex: hex(event),
            });
            pos += event.len();
        }
        if usize::from(id) >= initial_count {
            return Err("场景 ID 超过初始化表数量".into());
        }
        let initial_pointer = word(raw, initial + 1 + 2 * usize::from(id))?;
        let initial_at = offset(raw, initial_pointer, 43)?;
        let mut conditional_writes = Vec::new();
        let mut cursor = initial_at + 43;
        while *raw.get(cursor).ok_or("初始化规则缺少 FF")? != 255 {
            let v = raw.get(cursor..cursor + 4).ok_or("初始化规则截断")?;
            conditional_writes.push([v[0], v[1], v[2], v[3]]);
            cursor += 4;
        }
        scenes.push(Scene {
            id,
            event_pointer: pointer,
            events,
            initial_pointer,
            initial_states: raw[initial_at..initial_at + 43].to_vec(),
            conditional_writes,
        });
        at += 3;
    }
    Ok(Resource {
        name: file.name.clone(),
        side: file.side,
        file_index: file.index,
        header_words,
        texts,
        scenes,
    })
}

fn relocate_texts(raw: &[u8], resource: &Resource, streams: &[Vec<u8>]) -> Result<Vec<u8>> {
    let mut regions = resource
        .texts
        .iter()
        .map(|t| (t.file_offset, t.file_offset + t.source_bytes))
        .collect::<Vec<_>>();
    regions.sort();
    let mut arena: Vec<(usize, usize)> = Vec::new();
    for (start, end) in regions {
        if let Some(last) = arena.last_mut() {
            if start < last.1 {
                return Err("主文本存在共享／重叠槽，需先确认附加引用".into());
            }
            if start == last.1 {
                last.1 = end;
                continue;
            }
        }
        arena.push((start, end));
    }
    if raw.len() < DATA_LIMIT {
        arena.push((raw.len(), DATA_LIMIT));
    }
    let mut payload = raw.to_vec();
    let mut locations: BTreeMap<Vec<u8>, usize> = BTreeMap::new();
    let mut order = (0..streams.len()).collect::<Vec<_>>();
    order.sort_by_key(|i| (std::cmp::Reverse(streams[*i].len()), *i));
    for index in order {
        let stream = &streams[index];
        if locations.contains_key(stream) {
            continue;
        }
        let region = arena
            .iter()
            .enumerate()
            .filter(|(_, (a, b))| b - a >= stream.len())
            .min_by_key(|(_, (a, b))| b - a)
            .map(|(i, _)| i);
        let Some(region) = region else {
            let free = arena.iter().map(|(a, b)| b - a).sum::<usize>();
            return Err(format!(
                "{}[{}] 需要连续 {} 字节，可用总量 {free}；DATA 不得覆盖 $DFF6 向量",
                resource.name,
                index,
                stream.len()
            )
            .into());
        };
        let start = arena[region].0;
        let end = start + stream.len();
        if payload.len() < end {
            payload.resize(end, 0);
        }
        payload[start..end].copy_from_slice(stream);
        arena[region].0 = end;
        locations.insert(stream.clone(), start);
    }
    for (text, stream) in resource.texts.iter().zip(streams) {
        let at = locations[stream];
        let address = u16::try_from(0xA200 + at)?;
        payload[text.pointer_file_offset..text.pointer_file_offset + 2]
            .copy_from_slice(&address.to_le_bytes());
    }
    Ok(payload)
}
