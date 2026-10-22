use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const STUB_PREFIX: &[u8] = &[
    0x5B, 0x83, 0xC3, 0x03, 0x0E, 0x1F, 0x8C, 0xC0, 0x05, 0x10, 0x00, 0x01, 0x47, 0x08, 0x01, 0x47,
    0x0C, 0x8B, 0x07, 0x8B, 0x4F, 0x02, 0x8B, 0x7F, 0x04, 0xBE, 0x11, 0x27, 0xF7, 0xE6, 0x05, 0x19,
    0x36, 0x8B, 0xD0, 0xD1, 0xC2, 0xD1, 0xC2, 0x26, 0x30, 0x35, 0x47,
];

#[derive(Debug, Clone)]
pub struct LayerInfo {
    pub entry_offset: usize,
    pub encrypted_offset: usize,
    pub encrypted_size: usize,
    pub gap_before_entry: usize,
    pub seed: u16,
    pub target_cs: u16,
    pub target_ip: u16,
    pub target_ss: u16,
    pub target_sp: u16,
}

#[derive(Debug, Clone)]
pub struct WrapperInfo {
    pub source_size: usize,
    pub output_size: usize,
    pub layers: Vec<LayerInfo>,
    pub final_cs: u16,
    pub final_ip: u16,
    pub final_ss: u16,
    pub final_sp: u16,
}

#[derive(Debug)]
pub struct PreparedFile {
    pub source: PathBuf,
    pub destination_name: OsString,
    pub bytes: Vec<u8>,
    pub info: WrapperInfo,
    pub source_sha256: String,
    pub output_sha256: String,
}

#[derive(Debug)]
pub struct PreparedBatch {
    pub files: Vec<PreparedFile>,
}

#[derive(Debug)]
pub struct WriteReport {
    pub output_root: PathBuf,
    pub files: usize,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

fn u16_at(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("缺少 0x{offset:X} 处的 u16"))?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn checked_add_signed(base: usize, displacement: i16) -> Result<usize, String> {
    if displacement >= 0 {
        base.checked_add(displacement as usize)
            .ok_or_else(|| "入口 call 目标溢出".to_owned())
    } else {
        base.checked_sub(displacement.unsigned_abs() as usize)
            .ok_or_else(|| "入口 call 目标越过文件起点".to_owned())
    }
}

fn mz_entry(bytes: &[u8]) -> Result<(usize, usize), String> {
    if bytes.len() < 0x20 || bytes.get(..2) != Some(b"MZ") {
        return Err("不是 MZ 可执行文件".to_owned());
    }
    if u16_at(bytes, 0x06)? != 0 {
        return Err("当前只支持无重定位项的 FAIRYTALE 自解密 OVL".to_owned());
    }
    let header_size = usize::from(u16_at(bytes, 0x08)?)
        .checked_mul(16)
        .ok_or_else(|| "MZ 头大小溢出".to_owned())?;
    if !(0x20..=bytes.len()).contains(&header_size) {
        return Err(format!("无效 MZ 头大小 0x{header_size:X}"));
    }
    let initial_ip = usize::from(u16_at(bytes, 0x14)?);
    let initial_cs = usize::from(u16_at(bytes, 0x16)?);
    let entry = header_size
        .checked_add(initial_cs.checked_mul(16).ok_or("入口段溢出")?)
        .and_then(|value| value.checked_add(initial_ip))
        .ok_or_else(|| "入口偏移溢出".to_owned())?;
    if entry >= bytes.len() {
        return Err(format!("入口 0x{entry:X} 越过文件末尾"));
    }
    Ok((header_size, entry))
}

fn decrypt_one_layer(source: &[u8]) -> Result<Option<(Vec<u8>, LayerInfo)>, String> {
    let (header_size, entry) = mz_entry(source)?;
    let Some(entry_bytes) = source.get(entry..entry + 6) else {
        return Ok(None);
    };
    if entry_bytes[0] != 0xE8 || &entry_bytes[3..6] != b"TC0" {
        return Ok(None);
    }
    let entry_bytes = source
        .get(entry..entry + 6)
        .ok_or_else(|| "自解密入口越过文件末尾".to_owned())?;
    let displacement = i16::from_le_bytes([entry_bytes[1], entry_bytes[2]]);
    let stub = checked_add_signed(entry + 3, displacement)?;
    if source.get(stub..stub + STUB_PREFIX.len()) != Some(STUB_PREFIX) {
        return Err(format!("0x{stub:X} 处的解密 stub 签名不匹配"));
    }

    let parameters = entry + 6;
    let seed = u16_at(source, parameters)?;
    let encrypted_size = usize::from(u16_at(source, parameters + 2)?);
    let destination_offset = usize::from(u16_at(source, parameters + 4)?);
    let target_ip = u16_at(source, parameters + 6)?;
    let target_cs = u16_at(source, parameters + 8)?;
    let target_sp = u16_at(source, parameters + 10)?;
    let target_ss = u16_at(source, parameters + 12)?;
    if destination_offset < 0x100 {
        return Err(format!(
            "解密目标 DI=0x{destination_offset:04X} 位于 PSP 内"
        ));
    }
    let encrypted_offset = header_size
        .checked_add(destination_offset - 0x100)
        .ok_or_else(|| "解密数据起点溢出".to_owned())?;
    let encrypted_end = encrypted_offset
        .checked_add(encrypted_size)
        .ok_or_else(|| "解密数据终点溢出".to_owned())?;
    if encrypted_end > entry || encrypted_end > source.len() {
        return Err(format!(
            "解密区间 0x{encrypted_offset:X}..0x{encrypted_end:X} 越过入口 0x{entry:X}"
        ));
    }

    let mut load_module = source[header_size..].to_vec();
    let start_in_module = encrypted_offset - header_size;
    let mut state = seed;
    for byte in &mut load_module[start_in_module..start_in_module + encrypted_size] {
        state = state.wrapping_mul(0x2711).wrapping_add(0x3619);
        let key = state.rotate_left(2).to_be_bytes()[0];
        *byte ^= key;
    }

    let output_size = 0x20usize
        .checked_add(load_module.len())
        .ok_or_else(|| "去壳文件大小溢出".to_owned())?;
    if output_size > usize::from(u16::MAX) * 512 {
        return Err("去壳文件超过 MZ 页数字段容量".to_owned());
    }
    let pages = output_size.div_ceil(512);
    let last_page = output_size % 512;
    let mut output = source[..0x20].to_vec();
    put_u16(&mut output, 0x02, last_page as u16);
    put_u16(&mut output, 0x04, pages as u16);
    put_u16(&mut output, 0x08, 2);
    put_u16(&mut output, 0x12, 0);
    put_u16(&mut output, 0x14, target_ip);
    put_u16(&mut output, 0x16, target_cs);
    put_u16(&mut output, 0x0E, target_ss);
    put_u16(&mut output, 0x10, target_sp);
    output.extend_from_slice(&load_module);

    Ok(Some((
        output,
        LayerInfo {
            entry_offset: entry,
            encrypted_offset,
            encrypted_size,
            gap_before_entry: entry - encrypted_end,
            seed,
            target_cs,
            target_ip,
            target_ss,
            target_sp,
        },
    )))
}

pub fn decrypt_wrapped_mz(source: &[u8]) -> Result<(Vec<u8>, WrapperInfo), String> {
    let mut output = source.to_vec();
    let mut layers = Vec::new();
    for _ in 0..1024 {
        let Some((next, layer)) = decrypt_one_layer(&output)? else {
            break;
        };
        output = next;
        layers.push(layer);
    }
    if layers.is_empty() {
        let (_, entry) = mz_entry(source)?;
        return Err(format!("0x{entry:X} 处不是已确认的 TC0 自解密入口"));
    }
    if layers.len() == 1024 {
        return Err("TC0 嵌套层数超过 1024，拒绝继续".to_owned());
    }
    let (final_cs, final_ip, final_ss, final_sp) = (
        u16_at(&output, 0x16)?,
        u16_at(&output, 0x14)?,
        u16_at(&output, 0x0E)?,
        u16_at(&output, 0x10)?,
    );
    Ok((
        output,
        WrapperInfo {
            source_size: source.len(),
            output_size: source.len() - (usize::from(u16_at(source, 0x08)?) * 16 - 0x20),
            layers,
            final_cs,
            final_ip,
            final_ss,
            final_sp,
        },
    ))
}

fn output_name(path: &Path) -> Result<OsString, String> {
    let mut name = path
        .file_name()
        .ok_or_else(|| format!("输入没有文件名: {}", path.display()))?
        .to_os_string();
    name.push(".decrypted.exe");
    Ok(name)
}

pub fn prepare_batch(inputs: &[PathBuf]) -> Result<PreparedBatch, String> {
    if inputs.is_empty() {
        return Err("至少需要一个 OVL 输入".to_owned());
    }
    let mut files = Vec::with_capacity(inputs.len());
    let mut names = std::collections::BTreeSet::new();
    for input in inputs {
        if !input.is_file() {
            return Err(format!("输入不是文件: {}", input.display()));
        }
        let source =
            fs::read(input).map_err(|error| format!("无法读取 {}: {error}", input.display()))?;
        let (bytes, info) = decrypt_wrapped_mz(&source)
            .map_err(|error| format!("无法解析 {}: {error}", input.display()))?;
        let destination_name = output_name(input)?;
        let folded = destination_name.to_string_lossy().to_ascii_lowercase();
        if !names.insert(folded) {
            return Err(format!(
                "多个输入会生成同名输出: {}",
                destination_name.to_string_lossy()
            ));
        }
        files.push(PreparedFile {
            source: input.clone(),
            destination_name,
            source_sha256: sha256(&source),
            output_sha256: sha256(&bytes),
            bytes,
            info,
        });
    }
    Ok(PreparedBatch { files })
}

pub fn validate_output_target(output: &Path, overwrite: bool) -> Result<(), String> {
    if output.exists() {
        if !output.is_dir() {
            return Err(format!("输出已存在且不是目录: {}", output.display()));
        }
        if !overwrite {
            return Err(format!(
                "输出目录已存在；请选择新目录或显式启用覆盖: {}",
                output.display()
            ));
        }
    } else if output.parent().is_none_or(|parent| !parent.is_dir()) {
        return Err(format!("输出父目录不存在: {}", output.display()));
    }
    Ok(())
}

fn temporary_path(output: &Path) -> PathBuf {
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("decrypted");
    output.with_file_name(format!(".{name}.partial-{}", std::process::id()))
}

pub fn write_prepared(
    prepared: PreparedBatch,
    output: &Path,
    overwrite: bool,
) -> Result<WriteReport, String> {
    validate_output_target(output, overwrite)?;
    let temporary = temporary_path(output);
    if temporary.exists() {
        return Err(format!("临时输出已存在: {}", temporary.display()));
    }
    fs::create_dir(&temporary)
        .map_err(|error| format!("无法创建 {}: {error}", temporary.display()))?;
    let result = (|| {
        let mut input_bytes = 0u64;
        let mut output_bytes = 0u64;
        for item in &prepared.files {
            let destination = temporary.join(&item.destination_name);
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)
                .map_err(|error| format!("无法创建 {}: {error}", destination.display()))?;
            file.write_all(&item.bytes)
                .map_err(|error| format!("无法写入 {}: {error}", destination.display()))?;
            input_bytes += item.info.source_size as u64;
            output_bytes += item.info.output_size as u64;
        }
        if output.exists() {
            fs::remove_dir_all(output)
                .map_err(|error| format!("无法替换 {}: {error}", output.display()))?;
        }
        fs::rename(&temporary, output)
            .map_err(|error| format!("无法提交 {}: {error}", output.display()))?;
        Ok(WriteReport {
            output_root: output.to_owned(),
            files: prepared.files.len(),
            input_bytes,
            output_bytes,
        })
    })();
    if result.is_err() && temporary.exists() {
        let _ = fs::remove_dir_all(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrapped_sample(plain: &[u8]) -> Vec<u8> {
        let header_size = 0x200usize;
        let entry = header_size + plain.len();
        let stub = entry + 0x14;
        let mut bytes = vec![0u8; stub + STUB_PREFIX.len()];
        bytes[..2].copy_from_slice(b"MZ");
        put_u16(&mut bytes, 0x08, 0x20);
        put_u16(&mut bytes, 0x14, 0);
        put_u16(&mut bytes, 0x16, (plain.len() / 16) as u16);
        bytes[header_size..entry].copy_from_slice(plain);
        bytes[entry] = 0xE8;
        put_u16(&mut bytes, entry + 1, (stub - (entry + 3)) as u16);
        bytes[entry + 3..entry + 6].copy_from_slice(b"TC0");
        put_u16(&mut bytes, entry + 6, 1);
        put_u16(&mut bytes, entry + 8, plain.len() as u16);
        put_u16(&mut bytes, entry + 10, 0x100);
        put_u16(&mut bytes, entry + 12, 0);
        put_u16(&mut bytes, entry + 14, 0);
        put_u16(&mut bytes, entry + 16, 0x80);
        put_u16(&mut bytes, entry + 18, 0x30);
        bytes[stub..stub + STUB_PREFIX.len()].copy_from_slice(STUB_PREFIX);
        let mut state = 1u16;
        for byte in &mut bytes[header_size..entry] {
            state = state.wrapping_mul(0x2711).wrapping_add(0x3619);
            *byte ^= state.rotate_left(2).to_be_bytes()[0];
        }
        bytes
    }

    #[test]
    fn decrypts_tc0_wrapper_and_rebuilds_small_header() {
        let plain = vec![0x90; 0x100];
        let source = wrapped_sample(&plain);
        let (output, info) = decrypt_wrapped_mz(&source).unwrap();
        assert_eq!(&output[0x20..0x20 + plain.len()], plain);
        assert_eq!(u16_at(&output, 0x08).unwrap(), 2);
        assert_eq!(u16_at(&output, 0x14).unwrap(), 0);
        assert_eq!(u16_at(&output, 0x16).unwrap(), 0);
        assert_eq!(info.layers.len(), 1);
        assert_eq!(info.layers[0].encrypted_size, plain.len());
    }

    #[test]
    fn rejects_unknown_entry_stub() {
        let mut source = wrapped_sample(&[0x90; 0x100]);
        source[0x300] = 0x90;
        assert!(decrypt_wrapped_mz(&source).is_err());
    }
}
