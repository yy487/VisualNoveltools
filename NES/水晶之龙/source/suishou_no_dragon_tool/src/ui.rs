use crate::{
    fds::File,
    game::{Resource, Text},
    hex,
    text::{terminated, Codec},
    Result,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Locator {
    System {
        index: u8,
    },
    ItemName {
        item: u8,
    },
    ItemVariant {
        item: u8,
        state: u8,
        name_cpu: u16,
    },
    Engine {
        index: u8,
        error_code_cpu: Option<u16>,
    },
}

pub fn parse(file: &File, raw: &[u8], codec: &Codec) -> Result<Vec<Resource>> {
    if file.name != "FIXDAT" || file.load_address != 0x9000 || raw.len() < 0x780 {
        return Err("FIXDAT 界面表布局不符".into());
    }
    let mut system = Resource {
        name: "SYSTEM".into(),
        side: file.side,
        file_index: file.index,
        header_words: [0; 7],
        texts: Vec::new(),
        scenes: Vec::new(),
    };
    for (i, cpu) in [0x936Cu16, 0x9381, 0x938B, 0x9391, 0x9397, 0x93A4]
        .into_iter()
        .enumerate()
    {
        let at = usize::from(cpu - 0x9000);
        let bytes = terminated(raw, at)?;
        system.texts.push(Text {
            index: i,
            cpu_address: cpu,
            file_offset: at,
            pointer_file_offset: 0,
            raw_hex: hex(bytes),
            source_bytes: bytes.len(),
            original: codec.decode(bytes)?,
            ui: Some(Locator::System { index: i as u8 }),
            ui_reference: None,
        });
    }
    let mut items = Resource {
        name: "ITEMS".into(),
        side: file.side,
        file_index: file.index,
        header_words: [0; 7],
        texts: Vec::new(),
        scenes: Vec::new(),
    };
    let mut at = 0x3F7;
    for item in 0..15u8 {
        let length = usize::from(*raw.get(at).ok_or("道具表截断")?);
        let end = at + length;
        if length < 5 || end > 0x780 {
            return Err("道具记录长度非法".into());
        }
        let name_at = at + 2;
        let name_bytes = terminated(&raw[..end], name_at)?;
        let name = codec.decode(name_bytes)?;
        items.texts.push(Text {
            index: items.texts.len(),
            cpu_address: (0x9000 + name_at) as u16,
            file_offset: name_at,
            pointer_file_offset: 0,
            raw_hex: hex(name_bytes),
            source_bytes: name_bytes.len(),
            original: name.clone(),
            ui: Some(Locator::ItemName { item }),
            ui_reference: None,
        });
        let mut pos = name_at + name_bytes.len();
        let mut states = std::collections::BTreeSet::new();
        while *raw.get(pos).ok_or("道具状态表截断")? != 255 {
            let state = raw[pos];
            if !states.insert(state) {
                return Err("道具状态编号重复".into());
            }
            pos += 1;
            let bytes = terminated(&raw[..end], pos)?;
            let cpu = (0x9000 + pos) as u16;
            items.texts.push(Text {
                index: items.texts.len(),
                cpu_address: cpu,
                file_offset: pos,
                pointer_file_offset: 0,
                raw_hex: hex(bytes),
                source_bytes: bytes.len(),
                original: format!("{name}{}", codec.decode(bytes)?),
                ui: Some(Locator::ItemVariant {
                    item,
                    state,
                    name_cpu: (0x9000 + name_at) as u16,
                }),
                ui_reference: None,
            });
            pos += bytes.len();
        }
        if pos + 1 != end {
            return Err("道具长度和状态表终止符不符".into());
        }
        at = end;
    }
    if at != 0x765 || items.texts.len() != 35 {
        return Err("道具表未覆盖 15 个名称和 20 个状态".into());
    }
    Ok(vec![system, items])
}

pub fn validate_edit(text: &Text, message: &str) -> Result<()> {
    if text.original != message && matches!(text.ui, Some(Locator::Engine { .. })) {
        return Err(
            "磁盘错误与存档确认记录已提取；当前保持只读，避免修改固定布局和运行时错误码".into(),
        );
    }
    if text.original != message && matches!(text.ui, Some(Locator::System { index: 2..=4 })) {
        return Err("LOAD/SAVE 固定布局标签及空白控制记录已提取，但当前保持只读；游戏存档界面仍是独立未完成项".into());
    }
    Ok(())
}

pub fn parse_engine(file: &File, raw: &[u8], codec: &Codec) -> Result<Resource> {
    if file.name != "ADVS" || file.load_address != 0x6000 || raw.len() != 0x2A00 {
        return Err("ADVS 系统文字表布局不符".into());
    }
    let mut result = Resource {
        name: "ENGINE".into(),
        side: file.side,
        file_index: file.index,
        header_words: [0; 7],
        texts: Vec::new(),
        scenes: Vec::new(),
    };
    for index in 0..10u8 {
        let cpu = if index < 9 {
            0x6BD6 + u16::from(index) * 21
        } else {
            0x790A
        };
        let at = usize::from(cpu - 0x6000);
        let bytes = terminated(raw, at)?;
        if index < 9 && bytes.len() != 21 || index == 9 && bytes != b"OK.\xFF" {
            return Err("ADVS 错误表或确认文字范围不符".into());
        }
        let mut message = codec.decode(bytes)?;
        let dynamic = if index == 8 {
            if !message.ends_with("ERR.00") {
                return Err("可变错误码模板不符".into());
            }
            message.truncate(message.len() - 2);
            message.push_str("{error_code}");
            Some(0x6C8F)
        } else {
            None
        };
        result.texts.push(Text {
            index: index as usize,
            cpu_address: cpu,
            file_offset: at,
            pointer_file_offset: 0,
            raw_hex: hex(bytes),
            source_bytes: bytes.len(),
            original: message,
            ui: Some(Locator::Engine {
                index,
                error_code_cpu: dynamic,
            }),
            ui_reference: None,
        });
    }
    Ok(result)
}
