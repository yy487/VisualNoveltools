use crate::common::{
    decode_with, encode_with, read_json, write_json, EncodingChoice, Entry, Result,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct TobFile {
    bytes: Vec<u8>,
    text_start: usize,
    offset_start: usize,
    offsets: Vec<u32>,
    boundaries: Vec<u32>,
    code_end: usize,
    labels: Vec<LabelRef>,
}
#[derive(Debug, Clone)]
struct LabelRef {
    value_pos: usize,
    value: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Span {
    start: usize,
    end: usize,
}
#[derive(Debug, Clone)]
struct Param {
    outer_kind: u8,
    subtype: Option<u8>,
    span: Option<Span>,
    value_pos: Option<usize>,
    value_width: Option<usize>,
    len_pos: Option<usize>,
}
#[derive(Debug, Clone)]
struct Command {
    end: usize,
    opcode: u32,
    size_field: usize,
    jump_field: Option<(usize, usize)>,
    params: Vec<Param>,
}
#[derive(Debug, Clone, Default)]
struct ChunkInfo {
    spans: Vec<Span>,
    name: Option<Span>,
    selection: Option<Vec<Span>>,
    commands: Vec<Command>,
    code_boundaries: Vec<usize>,
}
#[derive(Debug, Clone)]
struct Replacement {
    span: Span,
    bytes: Vec<u8>,
}

fn u16_at(b: &[u8], p: usize) -> Option<u16> {
    b.get(p..p + 2).map(|x| u16::from_le_bytes([x[0], x[1]]))
}
fn u32_at(b: &[u8], p: usize) -> Option<u32> {
    b.get(p..p + 4)
        .map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
}
fn put_u32(b: &mut [u8], p: usize, v: u32) {
    b[p..p + 4].copy_from_slice(&v.to_le_bytes());
}
fn put_u16(b: &mut [u8], p: usize, v: u16) {
    b[p..p + 2].copy_from_slice(&v.to_le_bytes());
}

fn parse(data: Vec<u8>) -> Result<TobFile> {
    if data.len() < 16 || (&data[..4] != b"TOB0" && &data[..4] != b"TOB1") {
        return Err("not a TOB0/TOB1 file or file is truncated".into());
    }
    let tob1 = &data[..4] == b"TOB1";
    let label_size = u32_at(&data, 4).ok_or("truncated TOB label header")? as usize;
    let offset_start = 4usize
        .checked_add(label_size)
        .ok_or("TOB label section overflows")?;
    if label_size < 4 || offset_start > data.len() {
        return Err("invalid TOB label section size".into());
    }
    let label_count = u32_at(&data, 8).ok_or("truncated TOB label count")? as usize;
    let mut labels = Vec::with_capacity(label_count);
    let mut p = 12usize;
    for _ in 0..label_count {
        let size = *data.get(p).ok_or("truncated TOB label record")? as usize;
        if size < 5 || p + 1 + size > offset_start {
            return Err("invalid TOB label record size".into());
        }
        let value_pos = p + 1 + size - 4;
        if value_pos == p + 1 || data.get(value_pos - 1) != Some(&0) {
            return Err("TOB label name is not NUL-terminated".into());
        }
        labels.push(LabelRef {
            value_pos,
            value: u32_at(&data, value_pos).ok_or("truncated TOB label target")?,
        });
        p += 1 + size;
    }
    if p != offset_start {
        return Err("TOB label table has trailing bytes".into());
    }
    let section_size = u32_at(&data, offset_start).ok_or("truncated TOB offset header")? as usize;
    if section_size < 12 || offset_start + section_size > data.len() {
        return Err("invalid TOB offset section size".into());
    }
    let count = u32_at(&data, offset_start + 4).ok_or("truncated TOB offset count")? as usize;
    if count == 0
        || 8usize
            .checked_add(count * 4)
            .ok_or("TOB offset count overflows")?
            > section_size
    {
        return Err("invalid TOB offset count".into());
    }
    let raw_base = offset_start + section_size;
    let text_start = if tob1 {
        raw_base
            .checked_sub(count.checked_mul(4).ok_or("TOB1 code base overflows")?)
            .ok_or("invalid TOB1 code base")?
    } else {
        raw_base
    };
    let mut offsets = Vec::with_capacity(count);
    for i in 0..count {
        offsets.push(u32_at(&data, offset_start + 8 + i * 4).ok_or("truncated TOB offset")?);
    }
    let payload_len = data
        .len()
        .checked_sub(text_start)
        .ok_or("invalid TOB code base")?;
    if offsets.iter().any(|offset| *offset as usize > payload_len) {
        return Err("TOB offset exceeds code payload".into());
    }
    if offsets.windows(2).any(|x| x[0] > x[1]) {
        return Err("TOB offsets are not monotonic".into());
    }
    let code_end = scan_code_end(&data[text_start..])?;
    let mut boundaries = Vec::with_capacity(offsets.len() + 2);
    boundaries.push(0);
    boundaries.extend(offsets.iter().copied());
    boundaries.push(u32::try_from(payload_len).map_err(|_| "TOB payload exceeds 4 GiB")?);
    boundaries.dedup();
    Ok(TobFile {
        bytes: data,
        text_start,
        offset_start,
        offsets,
        boundaries,
        code_end,
        labels,
    })
}

fn is_marker(data: &[u8], p: usize, limit: usize) -> bool {
    p + 1 < limit && data[p] == b'[' && (data[p + 1] == b' ' || data[p + 1] == b's')
}

fn expression_end(data: &[u8], start: usize, limit: usize) -> Result<usize> {
    if start + 6 > limit || &data[start..start + 2] != b"[s" {
        return Err(format!("invalid [s expression at 0x{start:x}"));
    }
    let len = u32_at(data, start + 2).ok_or("truncated [s block length")? as usize;
    let flag = start
        .checked_add(6)
        .and_then(|x| x.checked_add(len))
        .ok_or("[s block overflows")?;
    if flag + 3 > limit {
        return Err(format!("[s block at 0x{start:x} exceeds chunk"));
    }
    let end = flag + 1 + u32_at(data, flag + 1).ok_or("truncated [s cache span")? as usize;
    if end > limit {
        return Err(format!("[s block at 0x{start:x} has invalid cache span"));
    }
    Ok(end)
}

fn cp932_unit_len(data: &[u8], p: usize, limit: usize) -> Result<usize> {
    let first = *data
        .get(p)
        .ok_or_else(|| format!("truncated text at 0x{p:x}"))?;
    if (0x81..=0x9f).contains(&first) || (0xe0..=0xfc).contains(&first) {
        let trail = *data
            .get(p + 1)
            .ok_or_else(|| format!("truncated CP932 lead byte at 0x{p:x}"))?;
        if p + 1 >= limit
            || !((0x40..=0x7e).contains(&trail) || (0x80..=0xfc).contains(&trail))
            || trail == 0x7f
        {
            return Err(format!("invalid CP932 trail byte at 0x{p:x}"));
        }
        Ok(2)
    } else {
        Ok(1)
    }
}

fn scan_code_end(data: &[u8]) -> Result<usize> {
    let mut p = 0;
    while p < data.len() {
        if data[p] == 0 {
            return Ok(p);
        }
        if is_marker(data, p, data.len()) {
            p = if data[p + 1] == b's' {
                expression_end(data, p, data.len())?
            } else {
                parse_command(data, p, data.len())?.end
            };
        } else {
            p += cp932_unit_len(data, p, data.len())?;
        }
    }
    Ok(data.len())
}

fn parse_param(data: &[u8], start: usize, limit: usize) -> Result<(usize, Param)> {
    let kind = *data.get(start).ok_or("truncated TOB parameter kind")?;
    if kind == 0 {
        return Ok((
            start + 1,
            Param {
                outer_kind: kind,
                subtype: None,
                span: None,
                value_pos: None,
                value_width: None,
                len_pos: None,
            },
        ));
    }
    if kind == 2 {
        let end = start + 5;
        if end > limit || u32_at(data, start + 1).is_none() {
            return Err("truncated TOB immediate parameter".into());
        }
        return Ok((
            end,
            Param {
                outer_kind: kind,
                subtype: None,
                span: None,
                value_pos: Some(start + 1),
                value_width: Some(4),
                len_pos: None,
            },
        ));
    }
    let subtype = *data
        .get(start + 1)
        .ok_or("truncated TOB parameter subtype")?;
    match subtype {
        0 => {
            let end = start + 6;
            if end > limit || u32_at(data, start + 2).is_none() {
                return Err("truncated TOB immediate parameter".into());
            }
            Ok((
                end,
                Param {
                    outer_kind: kind,
                    subtype: Some(subtype),
                    span: None,
                    value_pos: Some(start + 2),
                    value_width: Some(4),
                    len_pos: None,
                },
            ))
        }
        1 => {
            let len = u16_at(data, start + 2).ok_or("truncated TOB string length")? as usize;
            let data_start = start + 4;
            let end = data_start.checked_add(len).ok_or("TOB string overflows")?;
            if end > limit {
                return Err("invalid TOB string parameter length".into());
            }
            let span = if len > 0 {
                let span_end = data[data_start..end]
                    .iter()
                    .position(|byte| *byte == 0)
                    .map(|index| data_start + index)
                    .unwrap_or(end);
                Some(Span {
                    start: data_start,
                    end: span_end,
                })
            } else {
                None
            };
            Ok((
                end,
                Param {
                    outer_kind: kind,
                    subtype: Some(subtype),
                    span,
                    value_pos: None,
                    value_width: None,
                    len_pos: Some(start + 2),
                },
            ))
        }
        2 => {
            let end = start + 10;
            if end > limit || u32_at(data, start + 2).is_none() || u32_at(data, start + 6).is_none()
            {
                return Err("truncated TOB table parameter".into());
            }
            Ok((
                end,
                Param {
                    outer_kind: kind,
                    subtype: Some(subtype),
                    span: None,
                    value_pos: None,
                    value_width: None,
                    len_pos: None,
                },
            ))
        }
        3 => {
            let l = u16_at(data, start + 2).ok_or("truncated TOB cache parameter")? as usize;
            let flag = start
                .checked_add(4)
                .and_then(|x| x.checked_add(l))
                .ok_or("TOB cache parameter overflows")?;
            let m = u16_at(data, flag + 1).ok_or("truncated TOB cache payload length")? as usize;
            let end = flag
                .checked_add(1)
                .and_then(|x| x.checked_add(m))
                .ok_or("TOB cache parameter overflows")?;
            if end > limit {
                return Err("invalid TOB cache parameter length".into());
            }
            Ok((
                end,
                Param {
                    outer_kind: kind,
                    subtype: Some(subtype),
                    span: None,
                    value_pos: None,
                    value_width: None,
                    len_pos: None,
                },
            ))
        }
        other => Err(format!("unknown TOB parameter subtype {other}")),
    }
}

fn parse_command(data: &[u8], start: usize, limit: usize) -> Result<Command> {
    if start + 7 > limit || &data[start..start + 2] != b"[ " {
        return Err(format!("invalid TOB command at 0x{start:x}"));
    }
    let opcode = u32_at(data, start + 2).ok_or("truncated TOB opcode")?;
    let mode = data[start + 6];
    let size_field = match mode {
        0 => start + 11,
        1 => start + 7,
        2 => start + 15,
        3 => {
            let rel =
                u16_at(data, start + 7).ok_or("truncated TOB condition cache offset")? as usize;
            let flag = start + 9 + rel;
            if flag + 3 > limit {
                return Err("TOB condition cache exceeds command".into());
            }
            flag + 1
                + u16_at(data, flag + 1).ok_or("truncated TOB condition cache length")? as usize
        }
        other => return Err(format!("unknown TOB condition layout {other}")),
    };
    let command_size = u32_at(data, size_field).ok_or("truncated TOB command size")? as usize;
    if command_size < 5 {
        return Err(format!("TOB command at 0x{start:x} has invalid size"));
    }
    let end = size_field
        .checked_add(command_size)
        .ok_or("TOB command size overflows")?;
    if end > limit {
        return Err(format!("TOB command at 0x{start:x} exceeds chunk"));
    }
    let count = data[size_field + 4] as usize;
    let mut params = Vec::with_capacity(count);
    let mut p = size_field + 5;
    for _ in 0..count {
        let (next, param) = parse_param(data, p, end)?;
        params.push(param);
        p = next;
    }
    if p != end {
        return Err(format!(
            "TOB command at 0x{start:x} size does not match parameters"
        ));
    }
    let jump_field = (opcode <= 3)
        .then(|| {
            params
                .first()
                .and_then(|param| param.value_pos.zip(param.value_width))
        })
        .flatten();
    Ok(Command {
        end,
        opcode,
        size_field,
        jump_field,
        params,
    })
}

fn next_marker(data: &[u8], start: usize, limit: usize) -> usize {
    let mut p = start;
    while p + 1 < limit {
        if is_marker(data, p, limit) {
            return p;
        }
        p += 1;
    }
    limit
}

fn decode_span(data: &[u8], span: Span, encoding: EncodingChoice) -> Option<String> {
    let (text, errors) = decode_with(encoding, data.get(span.start..span.end)?).ok()?;
    if errors
        || text
            .chars()
            .any(|c| c == '\0' || (c.is_control() && c != '\n' && c != '\r' && c != '\t'))
    {
        None
    } else {
        Some(text)
    }
}

fn parse_chunk(
    data: &[u8],
    start: usize,
    end: usize,
    encoding: EncodingChoice,
) -> Result<ChunkInfo> {
    let mut info = ChunkInfo::default();
    let mut p = start;
    while p < end {
        if is_marker(data, p, end) && data[p + 1] == b's' {
            info.code_boundaries.push(p);
            p = expression_end(data, p, end)?;
            continue;
        }
        if is_marker(data, p, end) {
            let command = parse_command(data, p, end)
                .map_err(|e| format!("{e} (chunk 0x{start:x}, command 0x{p:x})"))?;
            info.code_boundaries.push(p);
            let next = next_marker(data, command.end, end);
            if command.end < next {
                let span = Span {
                    start: command.end,
                    end: next,
                };
                if decode_span(data, span, encoding).is_some() {
                    info.spans.push(span);
                }
            }
            if command.opcode == 25 {
                info.name = command.params.first().and_then(|x| x.span);
            }
            if command.opcode == 13 {
                let options: Vec<Span> = command
                    .params
                    .iter()
                    .skip(1)
                    .filter_map(|x| x.span)
                    .collect();
                if !options.is_empty() {
                    info.selection = Some(options);
                }
            }
            info.commands.push(command);
            p = next;
        } else {
            p += 1;
        }
    }
    Ok(info)
}

fn replace_ranges(original: &[u8], replacements: &[Replacement]) -> Result<Vec<u8>> {
    let mut sorted = replacements.to_vec();
    sorted.sort_by_key(|r| r.span.start);
    let mut out = Vec::with_capacity(original.len());
    let mut cursor = 0;
    for r in sorted {
        if r.span.start < cursor || r.span.end < r.span.start || r.span.end > original.len() {
            return Err("overlapping or out-of-range TOB replacements".into());
        }
        out.extend_from_slice(&original[cursor..r.span.start]);
        out.extend_from_slice(&r.bytes);
        cursor = r.span.end;
    }
    out.extend_from_slice(&original[cursor..]);
    Ok(out)
}

fn map_position(pos: usize, replacements: &[Replacement]) -> Result<usize> {
    let mut delta: i64 = 0;
    for r in replacements {
        if pos < r.span.start {
            break;
        }
        if pos == r.span.start {
            break;
        }
        if pos < r.span.end {
            return Err(format!(
                "TOB reference points inside edited span at 0x{pos:x}"
            ));
        }
        delta += r.bytes.len() as i64 - (r.span.end - r.span.start) as i64;
    }
    usize::try_from(pos as i64 + delta).map_err(|_| "TOB relocated position underflows".into())
}

fn selection_replacement(
    spans: &[Span],
    text: &str,
    encoding: EncodingChoice,
) -> Result<Vec<Replacement>> {
    let body = text
        .strip_prefix("Select:")
        .or_else(|| text.strip_prefix("Ｓｅｌｅｃｔ:"))
        .ok_or("selection must start with Select:")?;
    let parts: Vec<&str> = body.split("|||||").collect();
    if parts.len() != spans.len() {
        return Err("selection option count cannot change".into());
    }
    spans
        .iter()
        .zip(parts)
        .map(|(span, value)| {
            Ok(Replacement {
                span: *span,
                bytes: encode_with(encoding, value)?,
            })
        })
        .collect()
}

fn split_text_bytes(encoded: &[u8], spans: &[Span]) -> Result<Vec<Vec<u8>>> {
    if spans.len() <= 1 {
        return Ok(vec![encoded.to_vec()]);
    }
    let total_source: usize = spans.iter().map(|span| span.end - span.start).sum();
    if total_source == 0 {
        return Err("cannot split text across empty source slots".into());
    }
    let mut character_boundaries = vec![0usize];
    let mut p = 0;
    while p < encoded.len() {
        p += cp932_unit_len(encoded, p, encoded.len())?;
        character_boundaries.push(p);
    }
    let mut cuts = vec![0usize];
    let mut source_prefix = 0usize;
    for span in spans.iter().take(spans.len() - 1) {
        source_prefix += span.end - span.start;
        let target = (encoded.len() * source_prefix + total_source / 2) / total_source;
        let cut = *character_boundaries
            .iter()
            .min_by_key(|candidate| candidate.abs_diff(target))
            .ok_or("cannot split encoded text")?;
        cuts.push(cut);
    }
    cuts.push(encoded.len());
    Ok(cuts
        .windows(2)
        .map(|cut| encoded[cut[0]..cut[1]].to_vec())
        .collect())
}

pub fn extract_file(
    input: &Path,
    output: &Path,
    file_name: &str,
    requested: EncodingChoice,
    overwrite: bool,
) -> Result<usize> {
    let file = parse(fs::read(input).map_err(|e| format!("read {}: {e}", input.display()))?)?;
    let payload = &file.bytes[file.text_start..];
    let mut entries = Vec::new();
    for (index, pair) in file.boundaries.windows(2).enumerate() {
        let start = pair[0] as usize;
        if start >= file.code_end {
            continue;
        }
        let end = (pair[1] as usize).min(file.code_end);
        let info = parse_chunk(payload, start, end, requested)?;
        if let Some(options) = info.selection.as_ref() {
            let values: Vec<String> = options
                .iter()
                .filter_map(|s| decode_span(payload, *s, requested))
                .collect();
            if values.len() == options.len() {
                let text = format!(
                    "Select:{}",
                    values
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .join("|||||")
                );
                entries.push(Entry {
                    file: file_name.into(),
                    index,
                    offset: start as u64,
                    entry_type: "selection".into(),
                    encoding: requested.label().into(),
                    name_index: None,
                    name: None,
                    scr_name: None,
                    scr_tag: None,
                    scr_msg: text.clone(),
                    message: text,
                });
            }
            continue;
        }
        if info.spans.is_empty() {
            continue;
        }
        let values: Vec<String> = info
            .spans
            .iter()
            .filter_map(|s| decode_span(payload, *s, requested))
            .collect();
        if values.len() != info.spans.len() {
            continue;
        }
        let message = values.concat();
        let name = info.name.and_then(|s| decode_span(payload, s, requested));
        entries.push(Entry {
            file: file_name.into(),
            index,
            offset: start as u64,
            entry_type: "text".into(),
            encoding: requested.label().into(),
            name_index: None,
            name: name.clone(),
            scr_name: name,
            scr_tag: None,
            scr_msg: message.clone(),
            message,
        });
    }
    write_json(output, &entries, overwrite)?;
    Ok(entries.len())
}

pub fn inject_file(
    input: &Path,
    translation: &Path,
    output: &Path,
    requested: EncodingChoice,
    overwrite: bool,
) -> Result<usize> {
    let file = parse(fs::read(input).map_err(|e| format!("read {}: {e}", input.display()))?)?;
    let entries = read_json(translation)?;
    let mut by_offset = HashMap::new();
    for entry in &entries {
        let off = usize::try_from(entry.offset).map_err(|_| "_offset overflow".to_string())?;
        if by_offset.insert(off, entry).is_some() {
            return Err(format!("duplicate TOB _offset 0x{off:x}"));
        }
    }
    let payload = &file.bytes[file.text_start..];
    let mut infos = Vec::with_capacity(file.boundaries.len().saturating_sub(1));
    let mut replacements = Vec::new();
    let mut length_updates: Vec<(usize, usize)> = Vec::new();
    let mut command_deltas: HashMap<usize, i64> = HashMap::new();
    let mut changed = 0;
    for (index, pair) in file.boundaries.windows(2).enumerate() {
        let start = pair[0] as usize;
        if start >= file.code_end {
            continue;
        }
        let end = (pair[1] as usize).min(file.code_end);
        let info = parse_chunk(payload, start, end, requested)?;
        let Some(entry) = by_offset.get(&start) else {
            infos.push(info);
            continue;
        };
        let enc = EncodingChoice::parse(&entry.encoding)?;
        if entry.entry_type == "selection" {
            let spans = info
                .selection
                .as_ref()
                .ok_or("selection entry has no opcode 13 options")?;
            let source = spans
                .iter()
                .map(|s| {
                    decode_span(payload, *s, requested)
                        .ok_or_else(|| "TOB selection source decode failed".to_string())
                })
                .collect::<Result<Vec<_>>>()?;
            let source = format!(
                "Select:{}",
                source
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join("|||||")
            );
            if source != entry.scr_msg {
                return Err(format!("TOB scr_msg mismatch at offset 0x{start:x}"));
            }
            if entry.message != entry.scr_msg {
                let option_replacements = selection_replacement(spans, &entry.message, enc)?;
                for replacement in &option_replacements {
                    if let Some((len_pos, size_field)) = info.commands.iter().find_map(|command| {
                        command.params.iter().find_map(|param| {
                            (param.span == Some(replacement.span))
                                .then_some((param.len_pos, command.size_field))
                        })
                    }) {
                        if let Some(len_pos) = len_pos {
                            length_updates.push((len_pos, replacement.bytes.len() + 1));
                        }
                        *command_deltas.entry(size_field).or_default() += replacement.bytes.len()
                            as i64
                            - replacement.span.end.saturating_sub(replacement.span.start) as i64;
                    }
                }
                replacements.extend(option_replacements);
                changed += 1;
            }
        } else {
            let source = info
                .spans
                .iter()
                .map(|s| {
                    decode_span(payload, *s, requested)
                        .ok_or_else(|| "TOB source decode failed".to_string())
                })
                .collect::<Result<Vec<_>>>()?
                .concat();
            if source != entry.scr_msg {
                return Err(format!("TOB scr_msg mismatch at offset 0x{start:x}"));
            }
            if entry.message != entry.scr_msg {
                let encoded = encode_with(enc, &entry.message)?;
                let parts = split_text_bytes(&encoded, &info.spans)?;
                for (span, bytes) in info.spans.iter().copied().zip(parts) {
                    replacements.push(Replacement { span, bytes });
                }
                changed += 1;
            }
            if let Some(name_span) = info.name {
                let source_name = decode_span(payload, name_span, requested)
                    .ok_or("TOB source name decode failed")?;
                if entry.scr_name.as_deref() != Some(source_name.as_str()) {
                    return Err(format!("TOB scr_name mismatch at offset 0x{start:x}"));
                }
                if let Some(name) = &entry.name {
                    if name != &source_name {
                        let encoded = encode_with(enc, name)?;
                        if let Some((len_pos, size_field)) =
                            info.commands.iter().find_map(|command| {
                                command.params.iter().find_map(|param| {
                                    (param.span == Some(name_span))
                                        .then_some((param.len_pos, command.size_field))
                                })
                            })
                        {
                            if let Some(len_pos) = len_pos {
                                length_updates.push((len_pos, encoded.len() + 1));
                            }
                            *command_deltas.entry(size_field).or_default() += encoded.len() as i64
                                - name_span.end.saturating_sub(name_span.start) as i64;
                        }
                        replacements.push(Replacement {
                            span: name_span,
                            bytes: encoded,
                        });
                        changed += 1;
                    }
                }
            }
        }
        infos.push(info);
        let _ = index;
    }
    if output.exists() && !overwrite {
        return Err(format!(
            "output exists: {} (use --overwrite)",
            output.display()
        ));
    }
    replacements.sort_by_key(|r| r.span.start);
    let mut rebuilt_payload = replace_ranges(payload, &replacements)?;
    for (old_pos, new_len) in length_updates {
        let pos = map_position(old_pos, &replacements)?;
        let value =
            u16::try_from(new_len).map_err(|_| "TOB string parameter exceeds 65535 bytes")?;
        put_u16(&mut rebuilt_payload, pos, value);
    }
    let mut new_offsets = Vec::with_capacity(file.offsets.len());
    for old in &file.offsets {
        new_offsets.push(
            u32::try_from(map_position(*old as usize, &replacements)?)
                .map_err(|_| "TOB offset exceeds 4 GiB")?,
        );
    }

    let mut jumps = Vec::new();
    let mut proven: HashSet<usize> = file
        .boundaries
        .iter()
        .copied()
        .map(|value| value as usize)
        .collect();
    for label in &file.labels {
        proven.insert(label.value as usize);
    }
    for info in &infos {
        proven.extend(info.code_boundaries.iter().copied());
        for command in &info.commands {
            if let Some((field, width)) = command.jump_field {
                let target = match width {
                    1 => payload
                        .get(field)
                        .copied()
                        .ok_or("truncated TOB byte control transfer")?
                        as usize,
                    4 => u32_at(payload, field).ok_or("truncated TOB control transfer")? as usize,
                    _ => return Err("invalid TOB control transfer width".into()),
                };
                if target != u32::MAX as usize {
                    jumps.push((field, width, target));
                }
            }
        }
    }
    let mut typed_refs = Vec::new();
    for info in &infos {
        for command in &info.commands {
            for param in &command.params {
                if param.outer_kind != 0xff || param.subtype != Some(0) {
                    continue;
                }
                let Some(field) = param.value_pos else {
                    continue;
                };
                let target =
                    u32_at(payload, field).ok_or("truncated TOB typed code reference")? as usize;
                if proven.contains(&target) {
                    typed_refs.push((field, target));
                }
            }
        }
    }
    for (field, delta) in command_deltas {
        let nf = map_position(field, &replacements)?;
        let old_size = u32_at(payload, field).ok_or("truncated TOB command size")? as i64;
        let new_size = old_size
            .checked_add(delta)
            .ok_or("TOB command size overflows")?;
        let encoded = u32::try_from(new_size).map_err(|_| "TOB command size is invalid")?;
        put_u32(&mut rebuilt_payload, nf, encoded);
    }
    for (field, width, target) in jumps {
        let nf = map_position(field, &replacements)?;
        let nt = map_position(target, &replacements)?;
        match width {
            1 => {
                let value = u8::try_from(nt)
                    .map_err(|_| format!("TOB byte control target 0x{nt:x} exceeds 8-bit field"))?;
                *rebuilt_payload
                    .get_mut(nf)
                    .ok_or("relocated TOB byte control field is out of range")? = value;
            }
            4 => put_u32(
                &mut rebuilt_payload,
                nf,
                u32::try_from(nt).map_err(|_| "TOB control transfer exceeds 4 GiB")?,
            ),
            _ => return Err("invalid TOB control transfer width".into()),
        }
    }
    for (field, target) in typed_refs {
        let nf = map_position(field, &replacements)?;
        let nt = map_position(target, &replacements)?;
        put_u32(
            &mut rebuilt_payload,
            nf,
            u32::try_from(nt).map_err(|_| "TOB typed code reference exceeds 4 GiB")?,
        );
    }
    let mut out = file.bytes[..file.text_start].to_vec();
    for (i, value) in new_offsets.iter().copied().enumerate() {
        put_u32(&mut out, file.offset_start + 8 + i * 4, value);
    }
    out.extend_from_slice(&rebuilt_payload);
    for label in &file.labels {
        put_u32(
            &mut out,
            label.value_pos,
            u32::try_from(map_position(label.value as usize, &replacements)?)
                .map_err(|_| "TOB label exceeds 4 GiB")?,
        );
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(output, out).map_err(|e| format!("write {}: {e}", output.display()))?;
    Ok(changed)
}
