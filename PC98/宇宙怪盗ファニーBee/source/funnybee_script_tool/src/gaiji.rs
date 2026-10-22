use std::collections::BTreeMap;

const BUILTIN_GAIJI: &[u8] = include_bytes!("../assets/GAIJI.DAT");
const HEADER_SIZE: usize = 0x22;
const RECORD_SIZE: usize = 0x22;
const GLYPH_BYTES: usize = 0x20;
const EXPECTED_COUNT: usize = 0xBC;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug)]
pub struct GaijiTable {
    decoded: BTreeMap<[u8; 2], String>,
    reverse: Vec<(String, [u8; 2])>,
    pub source_sha256: String,
}

impl GaijiTable {
    pub fn builtin() -> Result<Self> {
        Self::from_data(BUILTIN_GAIJI)
    }

    pub fn from_data(data: &[u8]) -> Result<Self> {
        validate_layout(data)?;
        validate_layout(BUILTIN_GAIJI)?;
        if data != BUILTIN_GAIJI {
            return Err(format!(
                "GAIJI.DAT 结构可识别，但字形内容与内置 FunnyBee 基线不一致: expected={} actual={}",
                crate::sha256_hex(BUILTIN_GAIJI),
                crate::sha256_hex(data)
            ));
        }

        let mut decoded = BTreeMap::new();
        for index in 0..EXPECTED_COUNT {
            let offset = HEADER_SIZE + index * RECORD_SIZE;
            let cell = data[offset];
            let row = data[offset + 1];
            let pair = jis_to_sjis(row, cell)?;
            let display = recognized_display(pair)
                .map(str::to_string)
                .unwrap_or_else(|| private_use_character(index).to_string());
            if decoded.insert(pair, display).is_some() {
                return Err(format!(
                    "GAIJI.DAT 中出现重复码位 {:02X}{:02X}",
                    pair[0], pair[1]
                ));
            }
        }

        let mut reverse = decoded
            .iter()
            .map(|(pair, display)| (display.clone(), *pair))
            .collect::<Vec<_>>();
        reverse.sort_by(|left, right| {
            right
                .0
                .len()
                .cmp(&left.0.len())
                .then_with(|| left.1.cmp(&right.1))
        });
        for window in reverse.windows(2) {
            if window[0].0 == window[1].0 {
                return Err(format!("GAIJI Unicode 显示映射重复: {:?}", window[0].0));
            }
        }

        Ok(Self {
            decoded,
            reverse,
            source_sha256: crate::sha256_hex(data),
        })
    }

    pub fn decode_pair(&self, pair: [u8; 2]) -> Option<&str> {
        self.decoded.get(&pair).map(String::as_str)
    }

    pub fn match_display(&self, text: &str) -> Option<([u8; 2], usize)> {
        self.reverse
            .iter()
            .find(|(display, _)| text.starts_with(display))
            .map(|(display, pair)| (*pair, display.len()))
    }

    pub fn display_for_index(&self, index: usize) -> Option<&str> {
        let pair = pair_for_index(index).ok()?;
        self.decode_pair(pair)
    }
}

fn validate_layout(data: &[u8]) -> Result<()> {
    let count = read_u16(data, 0)? as usize;
    let height = read_u16(data, 2)? as usize;
    if count != EXPECTED_COUNT || height != 16 {
        return Err(format!(
            "不是 FunnyBee GAIJI.DAT: count={count} height={height}"
        ));
    }
    let expected = HEADER_SIZE
        .checked_add(count * RECORD_SIZE)
        .ok_or_else(|| "GAIJI.DAT 长度溢出".to_string())?;
    if data.len() != expected {
        return Err(format!(
            "GAIJI.DAT 长度不符: expected={expected} actual={}",
            data.len()
        ));
    }
    for index in 0..count {
        let offset = HEADER_SIZE + index * RECORD_SIZE;
        let expected_row = if index < 94 { 0x76 } else { 0x77 };
        let expected_cell = 0x21 + (index % 94) as u8;
        if data[offset] != expected_cell || data[offset + 1] != expected_row {
            return Err(format!(
                "GAIJI.DAT 第 {index} 项码位异常: {:02X}{:02X}",
                data[offset + 1],
                data[offset]
            ));
        }
        if data.get(offset + 2..offset + 2 + GLYPH_BYTES).is_none() {
            return Err(format!("GAIJI.DAT 第 {index} 项点阵越界"));
        }
    }
    Ok(())
}

fn pair_for_index(index: usize) -> Result<[u8; 2]> {
    if index >= EXPECTED_COUNT {
        return Err(format!("GAIJI 索引越界: {index}"));
    }
    let row = if index < 94 { 0x76 } else { 0x77 };
    let cell = 0x21 + (index % 94) as u8;
    jis_to_sjis(row, cell)
}

fn jis_to_sjis(row: u8, cell: u8) -> Result<[u8; 2]> {
    if !(0x21..=0x7E).contains(&row) || !(0x21..=0x7E).contains(&cell) {
        return Err(format!("无效 JIS 码位 {row:02X}{cell:02X}"));
    }
    let mut lead = ((row - 0x21) >> 1) + 0x81;
    if lead > 0x9F {
        lead = lead
            .checked_add(0x40)
            .ok_or_else(|| "JIS 转 Shift-JIS 首字节溢出".to_string())?;
    }
    let trail = if row & 1 != 0 {
        let mut value = cell + 0x1F;
        if value >= 0x7F {
            value += 1;
        }
        value
    } else {
        cell + 0x7E
    };
    Ok([lead, trail])
}

fn private_use_character(index: usize) -> char {
    char::from_u32(0xE000 + index as u32).expect("188 FunnyBee gaiji fit in BMP PUA")
}

fn recognized_display(pair: [u8; 2]) -> Option<&'static str> {
    match pair {
        [0xEB, 0xA7] => Some("┃"),
        [0xEC, 0x93] => Some("ウケ"),
        _ => None,
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("GAIJI.DAT 0x{offset:X} 缺少 u16"))?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_builtin_slots() {
        let table = GaijiTable::builtin().expect("builtin GAIJI.DAT");
        assert_eq!(table.decoded.len(), EXPECTED_COUNT);
        assert_eq!(table.decode_pair([0xEB, 0xA7]), Some("┃"));
        assert_eq!(table.decode_pair([0xEC, 0x93]), Some("ウケ"));
        assert_eq!(table.display_for_index(0), Some("\u{e000}"));
    }

    #[test]
    fn reverse_mapping_is_lossless() {
        let table = GaijiTable::builtin().expect("builtin GAIJI.DAT");
        assert_eq!(
            table.match_display("┃后续"),
            Some(([0xEB, 0xA7], "┃".len()))
        );
        assert_eq!(
            table.match_display("ウケ后续"),
            Some(([0xEC, 0x93], "ウケ".len()))
        );
        let pua = private_use_character(20).to_string();
        assert_eq!(
            table.match_display(&pua).map(|value| value.0),
            pair_for_index(20).ok()
        );
    }
}
