use crate::{
    fds::File,
    font,
    game::{self, Binding, Native},
    hex, json_bytes, sha256,
    text::{terminated, Codec},
    Result,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

pub const BANK: usize = 8192;
const DEMO_SHA: &str = "2f9078803e1f8e0aebac437f9aa517c59c7f9490d951e53ed0f16aeb967c8b12";
const SIGNATURE: &[u8] = b"SUIZH3\0\0";
const CODE: &[u8] = include_bytes!("../assets/banked_runtime.bin");
#[derive(Deserialize)]
struct Abi {
    base: usize,
    labels: BTreeMap<String, u16>,
    mapper: BTreeMap<String, u16>,
    boot_patch: usize,
    core_sha256: String,
    fixed_sha256: String,
    fixed_payload_sha256: String,
    previous_fixed_sha256: String,
    previous_core_sha256: String,
    resources: Vec<SeedResource>,
    core_patches: Vec<CodePatch>,
}
#[derive(Deserialize)]
struct CodePatch {
    offset: usize,
    old: Vec<u8>,
    new: Vec<u8>,
}
#[derive(Deserialize)]
struct SeedResource {
    name: String,
    side: usize,
    id: u8,
    load_address: u16,
    sha256: String,
}
fn abi() -> Abi {
    serde_json::from_str(include_str!("../assets/banked_abi.json")).expect("compiled ABI")
}
fn u16_at(raw: &[u8], at: usize) -> Result<u16> {
    let p = raw.get(at..at + 2).ok_or("16 位值截断")?;
    Ok(u16::from_le_bytes([p[0], p[1]]))
}
fn scalar(raw: &[u8]) -> Result<char> {
    char::from_u32(u32::from_le_bytes(raw.try_into()?))
        .ok_or_else(|| "非法 Unicode 字库映射".into())
}

#[derive(Clone)]
pub struct Nes {
    pub bytes: Vec<u8>,
    pub native: Native,
    pub glyphs: BTreeMap<char, [u8; 32]>,
    pub banks: usize,
    pub v3: bool,
    pub current_runtime: bool,
    ui_references: BTreeMap<u16, Vec<u8>>,
}
struct Page {
    logical: Vec<char>,
    cells: Vec<char>,
    layout: u16,
}

fn pages(message: &str) -> Result<Vec<Page>> {
    pages_with_icon(message, false)
}
fn pages_with_icon(message: &str, icon_indent: bool) -> Result<Vec<Page>> {
    pages_with_layout(message, if icon_indent { 0xC000 } else { 0 })
}
fn pages_with_layout(message: &str, layout: u16) -> Result<Vec<Page>> {
    if message
        .chars()
        .any(|c| (c.is_control() && c != '\n') || c as u32 > 0xFFFF)
    {
        return Err("译文只允许 BMP 字符和 LF 换行；拒绝 NUL、CR、TAB 和其他控制字符".into());
    }
    let mut result = Vec::new();
    let mut page = Page {
        logical: Vec::new(),
        cells: if layout != 0 {
            vec![' '; 3]
        } else {
            Vec::new()
        },
        layout,
    };
    for c in message.chars() {
        if page.cells.len() == 39 {
            result.push(page);
            page = Page {
                logical: Vec::new(),
                cells: if layout != 0 {
                    vec![' '; 3]
                } else {
                    Vec::new()
                },
                layout,
            };
        }
        if layout == 0xC000 && !page.cells.is_empty() && page.cells.len().is_multiple_of(13) {
            page.cells.extend([' '; 3]);
        }
        page.logical.push(c);
        if c == '\n' {
            let n = 13 - page.cells.len() % 13;
            page.cells.extend(std::iter::repeat_n(' ', n));
        } else {
            page.cells.push(c);
        }
    }
    result.push(page);
    if result.len() > u16::MAX as usize {
        return Err("文本超过 65535 页".into());
    }
    Ok(result)
}

impl Nes {
    /// Update only verified engine code; preserve text resources and all font banks.
    pub fn repair_runtime(&self) -> Result<game::Rebuild> {
        if self.v3 && self.current_runtime {
            return Ok(game::Rebuild {
                image: self.bytes.clone(),
                changed_entries: 0,
                grown_files: Vec::new(),
            });
        }
        let a = abi();
        let mut image = self.bytes.clone();
        let core = self
            .native
            .files
            .iter()
            .find(|f| f.name == "ADVS")
            .ok_or("缺少 ADVS")?;
        patch_core(
            &mut image[core.data_offset..core.data_offset + core.size],
            &a,
        );
        let fixed_payload = self
            .native
            .files
            .iter()
            .find(|f| f.name == "FIXDAT")
            .ok_or("缺少 FIXDAT")?;
        image[fixed_payload.data_offset + 0x780..fixed_payload.data_offset + fixed_payload.size]
            .copy_from_slice(include_bytes!("../assets/ram_runtime.bin"));
        let fixed_start = 16 + (self.banks - 1) * BANK;
        let mut fixed = image[fixed_start..].to_vec();
        install_fixed(&mut fixed, &a);
        let reg = fixed
            .windows(5)
            .position(|w| w == [0xA9, 0x80 | (self.banks as u8 - 1), 0x8D, 0x17, 0x51])
            .ok_or("固定 bank 初始化不符")?;
        fixed[reg + 1] = 0x80 | (self.banks as u8 - 1);
        image[fixed_start..].copy_from_slice(&fixed);
        let new = Self::parse(&image)?;
        if self
            .native
            .resources
            .iter()
            .map(|r| r.view())
            .collect::<Vec<_>>()
            != new
                .native
                .resources
                .iter()
                .map(|r| r.view())
                .collect::<Vec<_>>()
            || self.glyphs != new.glyphs
        {
            return Err("驱动修补改变了正文或字模".into());
        }
        Ok(game::Rebuild {
            image,
            changed_entries: 0,
            grown_files: Vec::new(),
        })
    }
    pub fn convert(fds: &[u8], bios: &[u8]) -> Result<Self> {
        if sha256(fds) != "4c33ec6a2499d6a7a41206ee69cda3da30b587ba515896e8f806f0e33635b4c2"
            || bios.len() != BANK
        {
            return Err("转换要求已确认的原始 FDS 和 8KB disksys.rom BIOS".into());
        }
        let game = game::Game::parse(fds)?;
        let a = abi();
        let mut prg = vec![255; 64 * BANK];
        let mut fixed = bios.to_vec();
        let mapper = include_bytes!("../assets/mapper_runtime.bin");
        fixed[0x1000..0x1000 + mapper.len()].copy_from_slice(mapper);
        for (at, target) in [
            (0x1F8, a.mapper["read_entry"]),
            (0x239, a.mapper["write_entry"]),
            (0x23C, a.mapper["write_entry"]),
        ] {
            put_jump(&mut fixed, at, 0x4C, target);
        }
        put_jump(&mut fixed, 0x237, 0x4C, 0xE23C);
        fixed[0x1FFA..0x1FFC].copy_from_slice(&0x75A4u16.to_le_bytes());
        fixed[0x1FFC..0x1FFE].copy_from_slice(&a.mapper["reset"].to_le_bytes());
        fixed[0x1FFE..].copy_from_slice(&a.mapper["irq"].to_le_bytes());
        let mut bank = 0;
        for (i, file) in game.disk.files.iter().enumerate() {
            let mut raw = game.disk.payload(file).to_vec();
            if file.name == "ADVS" {
                for patch in &a.core_patches {
                    if raw.get(patch.offset..patch.offset + patch.old.len())
                        != Some(patch.old.as_slice())
                    {
                        return Err("原始 ADVS 补丁校验失败".into());
                    }
                    raw[patch.offset..patch.offset + patch.new.len()].copy_from_slice(&patch.new);
                }
                put_jump(&mut raw, 0x683B - 0x6000, 0x4C, a.labels["dispatch"]);
                put_jump(&mut raw, 0x7612 - 0x6000, 0x20, a.labels["nmi_finish"]);
                raw[0x1615..0x1617].fill(0xEA);
                put_jump(&mut raw, 0x65EF - 0x6000, 0x4C, a.labels["item_entry"]);
            }
            if file.name == "FIXDAT" {
                raw.extend(include_bytes!("../assets/ram_runtime.bin"));
            }
            let count = raw.len().div_ceil(BANK);
            if bank + count > 32 {
                return Err("原始资源 bank 布局异常".into());
            }
            prg[bank * BANK..bank * BANK + raw.len()].copy_from_slice(&raw);
            let rec = &mut fixed[0x1D00 + i * 10..0x1D00 + (i + 1) * 10];
            rec[..4].copy_from_slice(&[file.side as u8, file.id, bank as u8, file.kind]);
            rec[4..6].copy_from_slice(&file.load_address.to_le_bytes());
            rec[6..8].copy_from_slice(&(raw.len() as u16).to_le_bytes());
            rec[8] = u8::from(file.side == 0 && file.id <= 15);
            rec[9] = 0;
            bank += count;
        }
        install_fixed(&mut fixed, &a);
        prg[63 * BANK..].copy_from_slice(&fixed);
        let mut bytes = b"NES\x1A\x20\x00\x52\x08\x00\x00\x90\x09\x00\x00\x00\x00".to_vec();
        bytes.extend(prg);
        let nes = Self::parse(&bytes)?;
        for (old, new) in game.native.resources.iter().zip(&nes.native.resources) {
            if old.view() != new.view() || old.scenes != new.scenes {
                return Err("FDS 转换后文本或场景变化".into());
            }
        }
        Ok(nes)
    }
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let a = abi();
        let header = bytes.get(..16).ok_or("NES 头截断")?;
        if &header[..4] != b"NES\x1A"
            || ![32, 64].contains(&header[4])
            || header[5..] != [0, 0x52, 8, 0, 0, 0x90, 9, 0, 0, 0, 0]
        {
            return Err("要求水晶之龙已验证的 NES 2.0 / MMC5 / 512KB 或 1MB PRG 布局".into());
        }
        let banks = usize::from(header[4]) * 2;
        if bytes.len() != 16 + banks * BANK {
            return Err("PRG 长度与 NES 头不符".into());
        }
        let fixed = &bytes[16 + (banks - 1) * BANK..];
        let v3 = &fixed[0x1E40..0x1E48] == SIGNATURE;
        if !v3 && sha256(bytes) != DEMO_SHA {
            return Err("未识别的引擎版本；仅支持已验证首句版和本工具 V3 成品".into());
        }
        let mut current_runtime = !v3;
        if v3 {
            let mut normalized = fixed.to_vec();
            let register = normalized
                .windows(5)
                .position(|b| b == [0xA9, 0x80 | (banks as u8 - 1), 0x8D, 0x17, 0x51])
                .ok_or("固定 bank 初始化损坏")?;
            normalized[register + 1] = 0xBF;
            normalized[0x1D00..0x1D00 + 260].fill(0);
            normalized[0x1E40..0x1E60].fill(0);
            let hash = sha256(&normalized);
            current_runtime = hash == a.fixed_sha256;
            if !current_runtime && hash != a.previous_fixed_sha256 {
                return Err("V3 固定驱动或 BIOS 已被修改".into());
            }
        }
        let mut files = Vec::new();
        let mut occupied = BTreeSet::new();
        let mut side_indices = [0usize; 2];
        for (i, seed) in a.resources.iter().enumerate() {
            let pos = 0x1D00 + i * 10;
            let rec = &fixed[pos..pos + 10];
            let bank = usize::from(rec[2]);
            let size = usize::from(u16_at(rec, 6)?);
            let load = u16_at(rec, 4)?;
            if usize::from(rec[0]) != seed.side
                || rec[1] != seed.id
                || load != seed.load_address
                || rec[3] > 2
                || size == 0
                || bank + size.div_ceil(BANK) > banks - 1
                || rec[8] != u8::from(seed.side == 0 && seed.id <= 15)
                || rec[9] != 0
            {
                return Err(format!("资源 {} 描述符非法", seed.name).into());
            }
            for n in bank..bank + size.div_ceil(BANK) {
                if n == 32 || !occupied.insert(n) {
                    return Err("资源 bank 重叠或侵占原首句字库 bank".into());
                }
            }
            let offset = 16 + bank * BANK;
            let raw = &bytes[offset..offset + size];
            let hash = sha256(raw);
            if seed.name == "ADVS" && v3 {
                if hash
                    != if current_runtime {
                        a.core_sha256.clone()
                    } else {
                        a.previous_core_sha256.clone()
                    }
                {
                    return Err("V3 ADVS 引擎不符".into());
                }
            } else if seed.name == "FIXDAT" && v3 && current_runtime {
                if hash != a.fixed_payload_sha256 {
                    return Err("FIXDAT 渲染器与当前引擎不符".into());
                }
            } else if !seed.name.starts_with("DATA") && hash != seed.sha256 {
                return Err(format!("资源 {} 与已验证布局不符", seed.name).into());
            }
            if seed.name.starts_with("DATA") && size > game::DATA_LIMIT {
                return Err("DATA 覆盖 $DFF6 向量".into());
            }
            let index = side_indices[seed.side];
            side_indices[seed.side] += 1;
            files.push(File {
                side: seed.side,
                index,
                number: index as u8,
                id: seed.id,
                name: seed.name.clone(),
                name_hex: hex(seed.name.as_bytes()),
                kind: rec[3],
                load_address: load,
                size,
                header_offset: 16 + (banks - 1) * BANK + pos,
                data_offset: offset,
                declared: true,
                sha256: hash,
            });
        }
        let macros = files
            .iter()
            .find(|f| f.name == "MSGMACRO")
            .ok_or("无字典")?;
        let codec =
            Codec::parse_macros(&bytes[macros.data_offset..macros.data_offset + macros.size])?;
        let mut glyphs = BTreeMap::new();
        let mut resources = Vec::new();
        let mut provisional = Self {
            bytes: bytes.to_vec(),
            native: Native {
                schema: 4,
                source_sha256: sha256(bytes),
                header_size: 16,
                files: files.clone(),
                sides: Vec::new(),
                dictionary_hex: codec.macros.iter().map(|m| hex(m)).collect(),
                resources: Vec::new(),
            },
            glyphs: BTreeMap::new(),
            banks,
            v3,
            current_runtime,
            ui_references: BTreeMap::new(),
        };
        for index in 0..10 {
            let f = files
                .iter()
                .find(|f| f.name == format!("DATA{index}"))
                .ok_or("缺少 DATA")?;
            let raw = &bytes[f.data_offset..f.data_offset + f.size];
            let r = game::parse_resource_with(f, raw, |at| {
                provisional
                    .resolve(raw, at, &codec)
                    .map(|(len, msg, _)| (len, msg))
            })?;
            for t in &r.texts {
                for (c, bits) in provisional.resolve(raw, t.file_offset, &codec)?.2 {
                    if let Some(old) = glyphs.insert(c, bits) {
                        if old != bits {
                            return Err(format!("字符 {c} 存在不一致字模").into());
                        }
                    }
                }
            }
            resources.push(r);
        }
        let mut ids = resources
            .iter()
            .flat_map(|r| r.scenes.iter().map(|s| s.id))
            .collect::<Vec<_>>();
        ids.sort();
        if ids != (0..51).collect::<Vec<_>>() {
            return Err("场景集合不完整".into());
        }
        let fixed_file = files
            .iter()
            .find(|f| f.name == "FIXDAT")
            .ok_or("缺少 FIXDAT")?;
        let mut interface = crate::ui::parse(fixed_file, provisional.payload(fixed_file), &codec)?;
        let keys = interface
            .iter()
            .flat_map(|r| r.texts.iter().map(|t| t.cpu_address))
            .collect::<BTreeSet<_>>();
        provisional.ui_references = provisional.read_ui_map(fixed, &keys)?;
        for r in &mut interface {
            for t in &mut r.texts {
                if let Some(stub) = provisional.ui_references.get(&t.cpu_address) {
                    let (_, message, found) = provisional.resolve(stub, 0, &codec)?;
                    t.original = message;
                    t.ui_reference = Some(hex(stub));
                    for (ch, bits) in found {
                        if let Some(old) = glyphs.insert(ch, bits) {
                            if old != bits {
                                return Err("界面字模与正文字模不一致".into());
                            }
                        }
                    }
                }
            }
        }
        resources.extend(interface);
        let engine_file = files.iter().find(|f| f.name == "ADVS").ok_or("缺少 ADVS")?;
        resources.push(crate::ui::parse_engine(
            engine_file,
            provisional.payload(engine_file),
            &codec,
        )?);
        let ui_references = provisional.ui_references;
        let native = Native {
            resources,
            ..provisional.native
        };
        Ok(Self {
            bytes: bytes.to_vec(),
            native,
            glyphs,
            banks,
            v3,
            current_runtime,
            ui_references,
        })
    }
    pub fn payload(&self, f: &File) -> &[u8] {
        &self.bytes[f.data_offset..f.data_offset + f.size]
    }
    fn read_ui_map(&self, fixed: &[u8], keys: &BTreeSet<u16>) -> Result<BTreeMap<u16, Vec<u8>>> {
        if !self.v3 {
            return Ok(BTreeMap::new());
        }
        let count = usize::from(u16_at(fixed, 0x1E4B)?);
        if count == 0 {
            return Ok(BTreeMap::new());
        }
        if !self.current_runtime || count > keys.len() {
            return Err("界面引用表版本或数量不符".into());
        }
        let bank = fixed[0x1E48];
        let address = u16_at(fixed, 0x1E49)?;
        if usize::from(address.checked_sub(0xC000).ok_or("界面表指针非法")?) + count * 12 > BANK
        {
            return Err("界面引用表跨 bank".into());
        }
        let table = self.bank_slice(bank, address, count * 12)?;
        let mut refs = BTreeMap::new();
        for rec in table.chunks_exact(12) {
            let key = u16_at(rec, 0)?;
            let stub = &rec[2..];
            if !keys.contains(&key)
                || !stub.starts_with(b"\x02ZH\x03")
                || stub[9] != 255
                || refs.insert(key, stub.to_vec()).is_some()
            {
                return Err("界面引用来源、格式或唯一性非法".into());
            }
        }
        Ok(refs)
    }
    fn bank_slice(&self, bank: u8, address: u16, length: usize) -> Result<&[u8]> {
        let offset = usize::from(address)
            .checked_sub(0xC000)
            .ok_or("ROM 指针低于 C000")?;
        if bank as usize >= self.banks - 1
            || offset >= BANK
            || offset + length > (self.banks - 1 - bank as usize) * BANK
        {
            return Err("bank 指针越界".into());
        }
        let start = 16 + bank as usize * BANK + offset;
        let first_bank = bank as usize;
        let end_bank = (offset + length).div_ceil(BANK) + first_bank;
        if self.native.files.iter().any(|f| {
            let a = (f.data_offset - 16) / BANK;
            let b = a + f.size.div_ceil(BANK);
            a < end_bank && b > first_bank
        }) {
            return Err("文本或字库引用侵占原资源 bank".into());
        }
        Ok(&self.bytes[start..start + length])
    }
    fn decode_page(
        &self,
        bank: u8,
        addr: u16,
        v3: bool,
    ) -> Result<(String, BTreeMap<char, [u8; 32]>)> {
        let head = self.bank_slice(bank, addr, 14)?;
        if &head[..8] != b"\x02ZH\x02\x01\x00\x00\x00" {
            return Err("页面不是单页 V2 数据包".into());
        }
        let n = usize::from(head[8]);
        let cells = usize::from(head[9]);
        if n == 0 || n > 40 || cells > 39 {
            return Err("字模或页面格数超限".into());
        }
        let packet = self.bank_slice(bank, addr, 15 + cells)?;
        if packet[14 + cells] != 255 || packet[14..14 + cells].iter().any(|b| usize::from(*b) >= n)
        {
            return Err("页面字符索引非法".into());
        }
        let mapaddr = u16_at(head, 12)?;
        let bitmapaddr = u16_at(head, 10)?;
        if usize::from(addr - 0xC000) + 15 + cells > BANK
            || usize::from(mapaddr.checked_sub(0xC000).ok_or("字库映射指针越界")?) + n * 4 > BANK
            || usize::from(bitmapaddr.checked_sub(0xC000).ok_or("字模指针越界")?) + n * 32 > BANK
        {
            return Err("页面或字库跨 bank".into());
        }
        let alphabet = self
            .bank_slice(bank, mapaddr, n * 4)?
            .chunks_exact(4)
            .map(scalar)
            .collect::<Result<Vec<_>>>()?;
        if alphabet[0] != ' ' || alphabet.iter().collect::<BTreeSet<_>>().len() != n {
            return Err("字库缺少首位空格或存在重复映射".into());
        }
        let bits = self.bank_slice(bank, bitmapaddr, n * 32)?;
        let glyphs = alphabet
            .iter()
            .zip(bits.chunks_exact(32))
            .map(|(c, b)| Ok((*c, b.try_into()?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let display = packet[14..14 + cells]
            .iter()
            .map(|b| alphabet[*b as usize])
            .collect::<String>();
        if !v3 {
            return Ok((display, glyphs));
        }
        let token_addr = addr.checked_add((15 + cells) as u16).ok_or("页面溢出")?;
        let token_flags = u16_at(self.bank_slice(bank, token_addr, 2)?, 0)?;
        let layout = token_flags & 0xC000;
        if layout == 0x4000 {
            return Err("未知页面排版标记".into());
        }
        let token_len = usize::from(token_flags & 0x3FFF);
        if token_len > 78 || usize::from(token_addr - 0xC000) + 2 + token_len > BANK {
            return Err("页面逻辑 token 数量异常".into());
        }
        let tokens = self.bank_slice(bank, token_addr + 2, token_len)?;
        let message = tokens
            .iter()
            .map(|b| {
                if *b == 254 {
                    Ok('\n')
                } else {
                    alphabet.get(*b as usize).copied().ok_or("逻辑字符索引越界")
                }
            })
            .collect::<std::result::Result<String, _>>()?;
        let check = pages_with_layout(&message, layout)?;
        if check.len() != 1 || check[0].cells.iter().collect::<String>() != display {
            return Err("逻辑换行与实际绘制数据不符".into());
        }
        Ok((message, glyphs))
    }
    fn resolve(
        &self,
        raw: &[u8],
        at: usize,
        codec: &Codec,
    ) -> Result<(usize, String, BTreeMap<char, [u8; 32]>)> {
        if raw.get(at..at + 4) == Some(b"\x02ZH\x03") {
            let stub = raw.get(at..at + 10).ok_or("V3 引用截断")?;
            if !self.v3 || stub[9] != 255 {
                return Err("V3 引用与引擎不符".into());
            }
            let count = usize::from(u16_at(stub, 7)?);
            if count == 0 {
                return Err("V3 页数为零".into());
            }
            let directory = self.bank_slice(stub[4], u16_at(stub, 5)?, count * 3)?;
            let mut message = String::new();
            let mut glyphs = BTreeMap::new();
            for record in directory.chunks_exact(3) {
                let (m, g) = self.decode_page(record[0], u16_at(record, 1)?, true)?;
                message.push_str(&m);
                for (c, b) in g {
                    if let Some(old) = glyphs.insert(c, b) {
                        if old != b {
                            return Err("跨页字模不一致".into());
                        }
                    }
                }
            }
            return Ok((10, message, glyphs));
        }
        if raw.get(at..at + 4) == Some(b"\x02ZH\x02") {
            let head = raw.get(at..at + 14).ok_or("V2 文本截断")?;
            if u16_at(head, 4)? != 1 {
                return Err("只支持已验证的首句 V2 数据包".into());
            }
            let n = head[8] as usize;
            let cells = head[9] as usize;
            let end = at + 15 + cells;
            let source = raw.get(at..end).ok_or("V2 索引截断")?;
            if source[end - at - 1] != 255 {
                return Err("V2 缺少 FF".into());
            }
            let map = self
                .bank_slice(32, u16_at(head, 12)?, n * 4)?
                .chunks_exact(4)
                .map(scalar)
                .collect::<Result<Vec<_>>>()?;
            let bits = self.bank_slice(32, u16_at(head, 10)?, n * 32)?;
            let glyphs = map
                .iter()
                .zip(bits.chunks_exact(32))
                .map(|(c, b)| Ok((*c, b.try_into()?)))
                .collect::<Result<BTreeMap<_, _>>>()?;
            let mut message = source[14..14 + cells]
                .iter()
                .map(|i| map.get(*i as usize).copied().ok_or("V2 索引越界"))
                .collect::<std::result::Result<String, _>>()?;
            let continuation = u16_at(head, 6)?;
            if continuation != 0 {
                let offset = usize::from(continuation)
                    .checked_sub(0xA200)
                    .ok_or("续文指针越界")?;
                message.push_str(&codec.decode(terminated(raw, offset)?)?);
            }
            return Ok((end - at, message, glyphs));
        }
        let source = terminated(raw, at)?;
        Ok((source.len(), codec.decode(source)?, BTreeMap::new()))
    }
    pub fn binding(&self) -> Result<Binding> {
        Ok(Binding {
            schema: 4,
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
            ("source.nes".into(), self.bytes.clone()),
            ("native.json".into(), json_bytes(&self.native)?),
            (
                "translations/binding.json".into(),
                json_bytes(&self.binding()?)?,
            ),
            ("fonts/atlas.png".into(), font::atlas(&self.glyphs)?),
            (
                "fonts/unicode.json".into(),
                json_bytes(&self.glyphs.keys().collect::<Vec<_>>())?,
            ),
        ]);
        for r in &self.native.resources {
            files.insert(
                format!("translations/{}.json", r.name),
                vn_text::write_template(&r.view())?,
            );
        }
        for f in &self.native.files {
            files.insert(
                format!("resources/{}.bin", f.name),
                self.payload(f).to_vec(),
            );
        }
        files.insert(
            "fonts/mono16.bin".into(),
            self.glyphs
                .values()
                .flat_map(|g| g.iter().copied())
                .collect(),
        );
        Ok(files)
    }
    pub fn import(
        &self,
        translations: &BTreeMap<String, Vec<vn_text::Entry>>,
        face: &str,
    ) -> Result<game::Rebuild> {
        for name in translations.keys() {
            if !self.native.resources.iter().any(|r| r.name == *name) {
                return Err(format!("未知翻译单元 {name}").into());
            }
        }
        let mut changed = 0;
        for r in &self.native.resources {
            if let Some(supplied) = translations.get(&r.name) {
                vn_text::validate_shape(&r.view(), supplied)?;
                for (t, e) in r.texts.iter().zip(supplied) {
                    crate::ui::validate_edit(t, &e.message)?;
                }
                changed += r
                    .view()
                    .iter()
                    .zip(supplied)
                    .filter(|(a, b)| a.message != b.message)
                    .count();
            }
        }
        if changed == 0 {
            return Ok(game::Rebuild {
                image: self.bytes.clone(),
                changed_entries: 0,
                grown_files: Vec::new(),
            });
        }
        let a = abi();
        let mut glyphs = self.glyphs.clone();
        let mut required = BTreeSet::from([' ']);
        for r in &self.native.resources {
            if let Some(supplied) = translations.get(&r.name) {
                for (t, e) in r.texts.iter().zip(supplied) {
                    if t.original != e.message {
                        pages(&e.message)?;
                        required.extend(e.message.chars().filter(|c| *c != '\n'));
                    }
                }
            }
        }
        let missing = required
            .iter()
            .filter(|c| !glyphs.contains_key(c))
            .copied()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            let raster = font::Raster::new(face)?;
            for c in missing {
                glyphs.insert(c, raster.glyph(c)?);
            }
        }
        let mut resources = self
            .native
            .files
            .iter()
            .map(|f| (f.name.clone(), self.payload(f).to_vec()))
            .collect::<BTreeMap<_, _>>();
        let mut prg = self.bytes[16..].to_vec();
        // Preserve old V3 page banks on incremental imports. Free-space reclamation is separate.
        let start = if self.v3 {
            (0..self.banks - 1)
                .rev()
                .find(|i| prg[i * BANK..(i + 1) * BANK].iter().any(|b| *b != 255))
                .unwrap_or(32)
                + 1
        } else {
            33
        };
        let mut allocator = Allocator {
            bytes: Vec::new(),
            start,
            offset: 0,
        };
        let mut grown = Vec::new();
        let mut ui_references = self.ui_references.clone();
        for r in &self.native.resources {
            let Some(entries) = translations.get(&r.name) else {
                continue;
            };
            if !r.name.starts_with("DATA") {
                for (t, e) in r.texts.iter().zip(entries) {
                    if t.original != e.message {
                        ui_references.insert(
                            t.cpu_address,
                            allocator.packet(&e.message, &glyphs, r.name == "ITEMS")?,
                        );
                    }
                }
                continue;
            }
            let raw = resources.get_mut(&r.name).ok_or("缺少 DATA")?;
            let before = raw.len();
            for (t, e) in r.texts.iter().zip(entries) {
                if t.original == e.message {
                    continue;
                }
                let stub = allocator.packet(&e.message, &glyphs, false)?;
                let at = if t.source_bytes >= 10
                    && !r.texts.iter().any(|other| {
                        other.index != t.index
                            && other.file_offset < t.file_offset + 10
                            && other.file_offset + other.source_bytes > t.file_offset
                    }) {
                    t.file_offset
                } else {
                    let at = raw.len();
                    raw.resize(at + 10, 0);
                    at
                };
                raw[at..at + 10].copy_from_slice(&stub);
                let ptr = u16::try_from(0xA200 + at)?;
                raw[t.pointer_file_offset..t.pointer_file_offset + 2]
                    .copy_from_slice(&ptr.to_le_bytes());
            }
            if raw.len() > game::DATA_LIMIT {
                return Err(format!("{} 无法再分配 10 字节引用槽", r.name).into());
            }
            if raw.len() != before {
                grown.push((r.name.clone(), before, raw.len()));
            }
        }
        let ui_map = if ui_references.is_empty() {
            None
        } else {
            let mut table = Vec::new();
            for (key, stub) in &ui_references {
                table.extend(key.to_le_bytes());
                table.extend(stub);
            }
            allocator.align_block(table.len())?;
            let (bank, address) = allocator.stream(&table)?;
            Some((bank, address, ui_references.len() as u16))
        };
        let required_banks = start + allocator.bytes.len().div_ceil(BANK) + 1;
        let banks = if required_banks <= 64 {
            64
        } else if required_banks <= 128 {
            128
        } else {
            return Err("新增 PRG 内容超过 MMC5 1MB 上限".into());
        };
        let mut fixed = prg[(self.banks - 1) * BANK..].to_vec();
        prg.resize(banks * BANK, 255);
        if banks > self.banks {
            prg[(self.banks - 1) * BANK..self.banks * BANK].fill(255);
        }
        prg[start * BANK..start * BANK + allocator.bytes.len()].copy_from_slice(&allocator.bytes);
        if !self.v3 || !self.current_runtime {
            let core = resources.get_mut("ADVS").ok_or("缺少 ADVS")?;
            patch_core(core, &a);
            let fixed_payload = resources.get_mut("FIXDAT").ok_or("缺少 FIXDAT")?;
            fixed_payload[0x780..].copy_from_slice(include_bytes!("../assets/ram_runtime.bin"));
            install_fixed(&mut fixed, &a);
        }
        fixed[0x1E48..0x1E4D].fill(0);
        if let Some((bank, address, count)) = ui_map {
            fixed[0x1E48] = bank;
            fixed[0x1E49..0x1E4B].copy_from_slice(&address.to_le_bytes());
            fixed[0x1E4B..0x1E4D].copy_from_slice(&count.to_le_bytes());
        }
        let reg = fixed
            .windows(5)
            .position(|w| w == [0xA9, 0x80 | (self.banks as u8 - 1), 0x8D, 0x17, 0x51])
            .ok_or("启动 bank 未找到")?;
        fixed[reg + 1] = 0x80 | (banks as u8 - 1);
        let mut next = 0;
        for (i, f) in self.native.files.iter().enumerate() {
            let raw = &resources[&f.name];
            let count = raw.len().div_ceil(BANK);
            if next <= 32 && next + count > 32 {
                next = 33;
            }
            if next + count > start {
                return Err("增长 DATA 需要重排 bank，但会侵占既有文本包".into());
            }
            prg[next * BANK..(next + count) * BANK].fill(255);
            prg[next * BANK..next * BANK + raw.len()].copy_from_slice(raw);
            let rec = &mut fixed[0x1D00 + i * 10..0x1D00 + (i + 1) * 10];
            rec[2] = next as u8;
            rec[6..8].copy_from_slice(&(raw.len() as u16).to_le_bytes());
            next += count;
        }
        prg[(banks - 1) * BANK..].copy_from_slice(&fixed);
        let mut image = self.bytes[..16].to_vec();
        image[4] = (banks / 2) as u8;
        image.extend(prg);
        let rebuilt = Self::parse(&image)?;
        for (old, new) in self.native.resources.iter().zip(&rebuilt.native.resources) {
            let expected = translations
                .get(&old.name)
                .cloned()
                .unwrap_or_else(|| old.view());
            if expected != new.view()
                || old.scenes != new.scenes
                || old.header_words != new.header_words
            {
                return Err(format!("{} 回注重提取或事件验证失败", old.name).into());
            }
        }
        Ok(game::Rebuild {
            image,
            changed_entries: changed,
            grown_files: grown,
        })
    }
    pub fn summary(&self) -> String {
        format!(
            "MMC5 / {}KB PRG / {} 主表文本 + 51 界面/引擎记录 / 51 场景 / V3 引擎修订 {}",
            self.banks * 8,
            self.native
                .resources
                .iter()
                .filter(|r| r.name.starts_with("DATA"))
                .map(|r| r.texts.len())
                .sum::<usize>(),
            if self.current_runtime && self.v3 {
                4
            } else if self.v3 {
                3
            } else {
                2
            }
        )
    }
}

fn install_fixed(fixed: &mut [u8], a: &Abi) {
    fixed[a.base - 0xE000..a.base - 0xE000 + CODE.len()].copy_from_slice(CODE);
    for (old, new) in [
        ("video_begin", "video_begin"),
        ("video_end", "video_end"),
        ("nmi_arm", "nmi_enter"),
    ] {
        put_jump(fixed, a.mapper[old] as usize - 0xE000, 0x4C, a.labels[new]);
    }
    put_jump(fixed, a.boot_patch, 0x4C, a.labels["boot_return"]);
    fixed[a.boot_patch + 3] = 0xEA;
    fixed[0x1E40..0x1E60].fill(0);
    fixed[0x1E40..0x1E48].copy_from_slice(SIGNATURE);
}
fn patch_core(core: &mut [u8], a: &Abi) {
    put_jump(core, 0x683B - 0x6000, 0x4C, a.labels["dispatch"]);
    put_jump(core, 0x7612 - 0x6000, 0x20, a.labels["nmi_finish"]);
    core[0x1615..0x1617].fill(0xEA);
    put_jump(core, 0x65EF - 0x6000, 0x4C, a.labels["item_entry"]);
}

fn put_jump(raw: &mut [u8], at: usize, op: u8, target: u16) {
    raw[at] = op;
    raw[at + 1..at + 3].copy_from_slice(&target.to_le_bytes());
}
struct Allocator {
    bytes: Vec<u8>,
    start: usize,
    offset: usize,
}
impl Allocator {
    fn align_block(&mut self, size: usize) -> Result<()> {
        if size > BANK {
            return Err("连续引用表超过单 bank".into());
        }
        if self.offset % BANK + size > BANK {
            let pad = BANK - self.offset % BANK;
            self.bytes.extend(std::iter::repeat_n(255, pad));
            self.offset += pad;
        }
        Ok(())
    }
    fn packet(
        &mut self,
        message: &str,
        glyphs: &BTreeMap<char, [u8; 32]>,
        icon_indent: bool,
    ) -> Result<Vec<u8>> {
        let split = pages_with_icon(message, icon_indent)?;
        let mut directory = Vec::new();
        for page in &split {
            let (bank, address) = self.page(page, glyphs)?;
            directory.push(bank);
            directory.extend(address.to_le_bytes());
        }
        let (bank, address) = self.stream(&directory)?;
        let mut stub = b"\x02ZH\x03".to_vec();
        stub.push(bank);
        stub.extend(address.to_le_bytes());
        stub.extend((split.len() as u16).to_le_bytes());
        stub.push(255);
        Ok(stub)
    }
    fn stream(&mut self, data: &[u8]) -> Result<(u8, u16)> {
        let bank = self.start + self.offset / BANK;
        if bank >= 127 {
            return Err("PRG bank 超限".into());
        }
        let addr = 0xC000 + (self.offset % BANK) as u16;
        self.bytes.extend(data);
        self.offset += data.len();
        Ok((bank as u8, addr))
    }
    fn page(&mut self, page: &Page, glyphs: &BTreeMap<char, [u8; 32]>) -> Result<(u8, u16)> {
        let mut alphabet = vec![' '];
        for c in &page.cells {
            if !alphabet.contains(c) {
                alphabet.push(*c);
            }
        }
        let size = 15 + page.cells.len() + 2 + page.logical.len() + alphabet.len() * 36;
        if self.offset % BANK + size > BANK {
            let pad = BANK - self.offset % BANK;
            self.bytes.extend(std::iter::repeat_n(255, pad));
            self.offset += pad;
        }
        let addr = 0xC000 + (self.offset % BANK) as u16;
        let map = addr + (15 + page.cells.len() + 2 + page.logical.len()) as u16;
        let bitmap = map + (alphabet.len() * 4) as u16;
        let mut blob = b"\x02ZH\x02\x01\x00\x00\x00".to_vec();
        blob.push(alphabet.len() as u8);
        blob.push(page.cells.len() as u8);
        blob.extend(bitmap.to_le_bytes());
        blob.extend(map.to_le_bytes());
        blob.extend(
            page.cells
                .iter()
                .map(|c| alphabet.iter().position(|v| v == c).expect("page alphabet") as u8),
        );
        blob.push(255);
        blob.extend(((page.logical.len() as u16) | page.layout).to_le_bytes());
        blob.extend(page.logical.iter().map(|c| {
            if *c == '\n' {
                254
            } else {
                alphabet
                    .iter()
                    .position(|v| v == c)
                    .expect("logical alphabet") as u8
            }
        }));
        for c in &alphabet {
            blob.extend((*c as u32).to_le_bytes());
        }
        for c in alphabet {
            blob.extend(glyphs[&c]);
        }
        self.stream(&blob)
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    #[test]
    fn item_text_reserves_icon_columns_on_every_row() -> Result<()> {
        let text = "钱。持有１００克雷迪特。通讯器。装在滑板车上，能够放大心灵感应。";
        let pages = pages_with_icon(text, true)?;
        assert!(pages.len() > 1);
        assert_eq!(
            pages
                .iter()
                .flat_map(|p| p.logical.iter())
                .collect::<String>(),
            text
        );
        for page in pages {
            for row in page.cells.chunks(13) {
                assert_eq!(&row[..3], &[' '; 3]);
            }
        }
        let old = pages_with_layout("123456789012345", 0x8000)?;
        assert_eq!(old[0].cells[13], '1');
        Ok(())
    }
}
