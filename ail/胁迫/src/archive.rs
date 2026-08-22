use crate::extract::prepare_output_dir;
use crate::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct Archive {
    entries: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryKind {
    Packed,
    Plain4,
    Plain6,
}

pub fn unpack_snl(input: &Path, output: &Path, overwrite: bool) -> Result<(usize, usize)> {
    prepare_output_dir(output)?;
    let data =
        fs::read(input).map_err(|err| format!("failed to read {}: {err}", input.display()))?;
    let archive = parse_archive(&data, &input.display().to_string())?;
    let mut written = 0usize;
    for (index, raw) in archive.entries.iter().enumerate() {
        if raw.is_empty() {
            continue;
        }
        let (_, payload) = decode_entry(raw, index)?;
        let path = output.join(format!("{index:04}.bin"));
        if path.exists() && !overwrite {
            return Err(format!(
                "{} already exists; pass --overwrite",
                path.display()
            ));
        }
        fs::write(&path, payload)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        written += 1;
    }
    Ok((archive.entries.len(), written))
}

pub fn pack_snl(
    source_snl: &Path,
    bins: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(usize, usize)> {
    if output.exists() && !overwrite {
        return Err(format!(
            "{} already exists; pass --overwrite",
            output.display()
        ));
    }
    let source_data = fs::read(source_snl)
        .map_err(|err| format!("failed to read {}: {err}", source_snl.display()))?;
    let mut archive = parse_archive(&source_data, &source_snl.display().to_string())?;
    let mut replaced = 0usize;
    for (index, raw) in archive.entries.iter_mut().enumerate() {
        if raw.is_empty() {
            continue;
        }
        let bin_path = bins.join(format!("{index:04}.bin"));
        if !bin_path.is_file() {
            continue;
        }
        let new_payload = fs::read(&bin_path)
            .map_err(|err| format!("failed to read {}: {err}", bin_path.display()))?;
        let (kind, original_payload) = decode_entry(raw, index)?;
        if new_payload == original_payload {
            continue;
        }
        *raw = encode_entry(kind, &new_payload)?;
        replaced += 1;
    }
    let rebuilt = build_archive(&archive)?;
    fs::write(output, rebuilt)
        .map_err(|err| format!("failed to write {}: {err}", output.display()))?;
    Ok((archive.entries.len(), replaced))
}

fn parse_archive(data: &[u8], label: &str) -> Result<Archive> {
    let count = read_u32(data, 0)? as usize;
    let header_len = 4usize
        .checked_add(
            count
                .checked_mul(4)
                .ok_or_else(|| format!("{label}: count overflow"))?,
        )
        .ok_or_else(|| format!("{label}: header overflow"))?;
    if header_len > data.len() {
        return Err(format!("{label}: truncated archive size table"));
    }
    let mut cursor = header_len;
    let mut entries = Vec::with_capacity(count);
    for index in 0..count {
        let size = read_u32(data, 4 + index * 4)? as usize;
        let end = cursor
            .checked_add(size)
            .ok_or_else(|| format!("{label}: entry {index} size overflow"))?;
        if end > data.len() {
            return Err(format!("{label}: entry {index} exceeds archive size"));
        }
        entries.push(data[cursor..end].to_vec());
        cursor = end;
    }
    if cursor != data.len() {
        return Err(format!(
            "{label}: archive table accounts for 0x{cursor:X} bytes, file has 0x{:X}",
            data.len()
        ));
    }
    Ok(Archive { entries })
}

fn build_archive(archive: &Archive) -> Result<Vec<u8>> {
    let count = u32::try_from(archive.entries.len())
        .map_err(|_| "archive has more than u32::MAX entries".to_string())?;
    let mut out = Vec::new();
    out.extend_from_slice(&count.to_le_bytes());
    for entry in &archive.entries {
        let size = u32::try_from(entry.len())
            .map_err(|_| "archive entry exceeds u32::MAX bytes".to_string())?;
        out.extend_from_slice(&size.to_le_bytes());
    }
    for entry in &archive.entries {
        out.extend_from_slice(entry);
    }
    Ok(out)
}

fn decode_entry(raw: &[u8], index: usize) -> Result<(EntryKind, Vec<u8>)> {
    if raw.len() < 4 {
        return Err(format!("archive entry {index} is shorter than 4 bytes"));
    }
    let signature = read_u32(raw, 0)?;
    let flag = signature as u16;
    if flag == 1 {
        if raw.len() < 6 {
            return Err(format!("packed archive entry {index} is truncated"));
        }
        let unpacked_len = read_u32(raw, 2)? as usize;
        let payload = lzss_decompress(&raw[6..], unpacked_len)?;
        Ok((EntryKind::Packed, payload))
    } else if signature == 0 || raw.get(4..8) == Some(b"OggS") {
        Ok((EntryKind::Plain4, raw[4..].to_vec()))
    } else {
        if raw.len() < 6 {
            return Err(format!("plain6 archive entry {index} is truncated"));
        }
        Ok((EntryKind::Plain6, raw[6..].to_vec()))
    }
}

fn encode_entry(kind: EntryKind, payload: &[u8]) -> Result<Vec<u8>> {
    let payload_len = u32::try_from(payload.len())
        .map_err(|_| "entry payload exceeds u32::MAX bytes".to_string())?;
    let mut out = Vec::new();
    match kind {
        EntryKind::Packed => {
            out.extend_from_slice(&1u16.to_le_bytes());
            out.extend_from_slice(&payload_len.to_le_bytes());
            out.extend_from_slice(&lzss_compress(payload));
        }
        EntryKind::Plain4 => {
            out.extend_from_slice(&[0, 0, 0, 0]);
            out.extend_from_slice(payload);
        }
        EntryKind::Plain6 => {
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&payload_len.to_le_bytes());
            out.extend_from_slice(payload);
        }
    }
    Ok(out)
}

fn lzss_decompress(input: &[u8], output_len: usize) -> Result<Vec<u8>> {
    let mut frame = vec![0x20u8; 0x1000];
    let mut frame_pos = 0x0FEEusize;
    let mut source = 0usize;
    let mut control = 0u16;
    let mut out = Vec::with_capacity(output_len);
    while out.len() < output_len {
        control >>= 1;
        if control & 0x100 == 0 {
            let byte = *input
                .get(source)
                .ok_or_else(|| "truncated LZSS control stream".to_string())?;
            source += 1;
            control = byte as u16 | 0xFF00;
        }
        if control & 1 == 0 {
            let byte = *input
                .get(source)
                .ok_or_else(|| "truncated LZSS literal".to_string())?;
            source += 1;
            out.push(byte);
            frame[frame_pos] = byte;
            frame_pos = (frame_pos + 1) & 0x0FFF;
        } else {
            let lo = *input
                .get(source)
                .ok_or_else(|| "truncated LZSS match".to_string())?;
            let hi = *input
                .get(source + 1)
                .ok_or_else(|| "truncated LZSS match".to_string())?;
            source += 2;
            let mut offset = lo as usize | (((hi & 0xF0) as usize) << 4);
            let length = (hi & 0x0F) as usize + 3;
            for _ in 0..length {
                if out.len() == output_len {
                    break;
                }
                let byte = frame[offset];
                offset = (offset + 1) & 0x0FFF;
                out.push(byte);
                frame[frame_pos] = byte;
                frame_pos = (frame_pos + 1) & 0x0FFF;
            }
        }
    }
    Ok(out)
}

fn lzss_compress(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() + input.len().div_ceil(8));
    for chunk in input.chunks(8) {
        out.push(0);
        out.extend_from_slice(chunk);
    }
    out
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| format!("u32 at 0x{offset:X} is out of range"))?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_lzss_round_trip() {
        for size in [0usize, 1, 7, 8, 9, 100, 4097] {
            let source: Vec<u8> = (0..size).map(|value| value as u8).collect();
            let packed = lzss_compress(&source);
            assert_eq!(lzss_decompress(&packed, source.len()).unwrap(), source);
        }
    }
}
