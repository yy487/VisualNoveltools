use crate::Result;

pub const MAGIC_TOB0: &[u8; 4] = b"TOB0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub name: Vec<u8>,
    pub target_pos: usize,
    pub target: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterKind {
    Null,
    Immediate,
    Dword,
    String,
    TableValue,
    Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub start: usize,
    pub end: usize,
    pub outer_kind: u8,
    pub inner_kind: Option<u8>,
    pub kind: ParameterKind,
    pub value_pos: Option<usize>,
    pub length_pos: Option<usize>,
    pub string: Option<std::ops::Range<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub start: usize,
    pub end: usize,
    pub opcode: u32,
    pub condition_kind: u8,
    pub size_pos: usize,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeRegion {
    Command(Command),
    Expression(Expression),
    Text(std::ops::Range<usize>),
    Terminator(usize),
}

impl CodeRegion {
    pub fn start(&self) -> usize {
        match self {
            Self::Command(value) => value.start,
            Self::Expression(value) => value.start,
            Self::Text(value) => value.start,
            Self::Terminator(value) => *value,
        }
    }

    pub fn end(&self) -> usize {
        match self {
            Self::Command(value) => value.end,
            Self::Expression(value) => value.end,
            Self::Text(value) => value.end,
            Self::Terminator(value) => value + 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TobFile {
    pub bytes: Vec<u8>,
    pub labels: Vec<Label>,
    pub offset_table_pos: usize,
    pub offset_target_positions: Vec<usize>,
    pub offsets: Vec<u32>,
    pub code_start: usize,
    pub regions: Vec<CodeRegion>,
    pub opaque_tail: std::ops::Range<usize>,
}

impl TobFile {
    pub fn code_len(&self) -> usize {
        self.opaque_tail.start - self.code_start
    }

    pub fn code_offset(&self, absolute: usize) -> Result<u32> {
        let relative = absolute
            .checked_sub(self.code_start)
            .ok_or_else(|| format!("absolute offset 0x{absolute:x} precedes TOB code"))?;
        u32::try_from(relative).map_err(|_| "TOB code offset exceeds 4 GiB".to_string())
    }

    pub fn command_at_code_offset(&self, target: u32) -> Option<&Command> {
        let absolute = self.code_start.checked_add(target as usize)?;
        self.regions.iter().find_map(|region| match region {
            CodeRegion::Command(command) if command.start == absolute => Some(command),
            _ => None,
        })
    }
}

pub fn parse(bytes: Vec<u8>) -> Result<TobFile> {
    if bytes.len() < 16 {
        return Err("TOB file is shorter than its fixed headers".to_string());
    }
    if bytes.get(..4) != Some(MAGIC_TOB0) {
        return Err(
            "unsupported TOB magic (only the observed TOB0 format is accepted)".to_string(),
        );
    }

    let label_section_size = read_u32(&bytes, 4, "label section size")? as usize;
    let offset_table_pos = 4usize
        .checked_add(label_section_size)
        .ok_or("label section offset overflow")?;
    if label_section_size < 8 || offset_table_pos > bytes.len() {
        return Err(format!(
            "invalid TOB label section size 0x{label_section_size:x}"
        ));
    }
    let label_count = read_u32(&bytes, 8, "label count")? as usize;
    let mut label_cursor = 12usize;
    let mut labels = Vec::with_capacity(label_count);
    for index in 0..label_count {
        let record_size = *bytes
            .get(label_cursor)
            .ok_or_else(|| format!("label {index} record header is truncated"))?
            as usize;
        if record_size < 5 {
            return Err(format!(
                "label {index} record is shorter than name + target"
            ));
        }
        let record_start = label_cursor + 1;
        let record_end = record_start
            .checked_add(record_size)
            .ok_or("label record offset overflow")?;
        if record_end > offset_table_pos {
            return Err(format!("label {index} record exceeds the label section"));
        }
        let target_pos = record_end - 4;
        let name = bytes[record_start..target_pos].to_vec();
        if name.last() != Some(&0) {
            return Err(format!("label {index} name is not NUL-terminated"));
        }
        labels.push(Label {
            name,
            target_pos,
            target: read_u32(&bytes, target_pos, "label target")?,
        });
        label_cursor = record_end;
    }
    if label_cursor != offset_table_pos {
        return Err(format!(
            "label records end at 0x{label_cursor:x}, expected 0x{offset_table_pos:x}"
        ));
    }

    let offset_section_size = read_u32(&bytes, offset_table_pos, "offset section size")? as usize;
    let offset_count = read_u32(&bytes, offset_table_pos + 4, "offset count")? as usize;
    let expected_offset_size = 8usize
        .checked_add(
            offset_count
                .checked_mul(4)
                .ok_or("offset table size overflow")?,
        )
        .ok_or("offset table size overflow")?;
    if offset_section_size != expected_offset_size {
        return Err(format!(
            "offset section size is 0x{offset_section_size:x}, expected 0x{expected_offset_size:x} for {offset_count} entries"
        ));
    }
    let code_start = offset_table_pos
        .checked_add(offset_section_size)
        .ok_or("code start overflow")?;
    if code_start > bytes.len() {
        return Err("offset table exceeds the file".to_string());
    }
    let mut offset_target_positions = Vec::with_capacity(offset_count);
    let mut offsets = Vec::with_capacity(offset_count);
    for index in 0..offset_count {
        let position = offset_table_pos + 8 + index * 4;
        offset_target_positions.push(position);
        offsets.push(read_u32(&bytes, position, "offset table target")?);
    }
    for (index, pair) in offsets.windows(2).enumerate() {
        if pair[0] >= pair[1] {
            return Err(format!(
                "offset table targets {index} and {} are not strictly increasing",
                index + 1
            ));
        }
    }
    let code_capacity = bytes.len() - code_start;
    for (index, target) in offsets.iter().enumerate() {
        if *target as usize >= code_capacity {
            return Err(format!(
                "offset table target {index} (0x{target:x}) exceeds the code"
            ));
        }
    }
    for (index, label) in labels.iter().enumerate() {
        if label.target != u32::MAX && label.target as usize >= code_capacity {
            return Err(format!(
                "label {index} target 0x{:x} exceeds the code",
                label.target
            ));
        }
    }

    let (regions, tail_start) = parse_code(&bytes, code_start)?;
    let file = TobFile {
        bytes,
        labels,
        offset_table_pos,
        offset_target_positions,
        offsets,
        code_start,
        regions,
        opaque_tail: tail_start..code_capacity + code_start,
    };
    validate_targets(&file)?;
    Ok(file)
}

fn parse_code(bytes: &[u8], code_start: usize) -> Result<(Vec<CodeRegion>, usize)> {
    let mut regions = Vec::new();
    let mut cursor = code_start;
    let mut text_start = None;
    while cursor < bytes.len() {
        if bytes[cursor] == 0 {
            flush_text(&mut regions, &mut text_start, cursor);
            regions.push(CodeRegion::Terminator(cursor));
            return Ok((regions, cursor + 1));
        }
        if bytes[cursor] != b'[' {
            text_start.get_or_insert(cursor);
            cursor += cp932_unit_len(bytes, cursor)?;
            continue;
        }
        flush_text(&mut regions, &mut text_start, cursor);
        match bytes.get(cursor + 1).copied() {
            Some(b' ') => {
                let command = parse_command(bytes, cursor)?;
                cursor = command.end;
                regions.push(CodeRegion::Command(command));
            }
            Some(b's') => {
                let expression = parse_expression(bytes, cursor)?;
                cursor = expression.end;
                regions.push(CodeRegion::Expression(expression));
            }
            Some(value) => {
                return Err(format!(
                    "unknown '[' sequence 0x{value:02x} at file offset 0x{:x}",
                    cursor + 1
                ));
            }
            None => return Err("truncated '[' at the end of TOB code".to_string()),
        }
    }
    flush_text(&mut regions, &mut text_start, cursor);
    Ok((regions, cursor))
}

fn flush_text(regions: &mut Vec<CodeRegion>, text_start: &mut Option<usize>, end: usize) {
    if let Some(start) = text_start.take() {
        if start < end {
            regions.push(CodeRegion::Text(start..end));
        }
    }
}

fn cp932_unit_len(bytes: &[u8], position: usize) -> Result<usize> {
    let first = bytes[position];
    if (0x81..=0x9f).contains(&first) || (0xe0..=0xfc).contains(&first) {
        let trail = *bytes
            .get(position + 1)
            .ok_or_else(|| format!("truncated CP932 lead byte at file offset 0x{position:x}"))?;
        if !((0x40..=0x7e).contains(&trail) || (0x80..=0xfc).contains(&trail)) || trail == 0x7f {
            return Err(format!(
                "invalid CP932 trail byte 0x{trail:02x} at file offset 0x{:x}",
                position + 1
            ));
        }
        Ok(2)
    } else {
        Ok(1)
    }
}

fn parse_expression(bytes: &[u8], start: usize) -> Result<Expression> {
    let expression_size = read_u32(bytes, start + 2, "[s expression size")? as usize;
    let cache_pos = checked_add(start + 6, expression_size, "[s cache position")?;
    let compiled_pos = cache_pos
        .checked_add(1)
        .ok_or("[s compiled block offset overflow")?;
    let compiled_size = read_u32(bytes, compiled_pos, "[s compiled block size")? as usize;
    if compiled_size < 4 {
        return Err(format!(
            "[s block at 0x{start:x} has a compiled size below 4"
        ));
    }
    let end = checked_add(compiled_pos, compiled_size, "[s end")?;
    if end > bytes.len() {
        return Err(format!("[s block at 0x{start:x} exceeds the file"));
    }
    Ok(Expression { start, end })
}

fn parse_command(bytes: &[u8], start: usize) -> Result<Command> {
    let opcode = read_u32(bytes, start + 2, "command opcode")?;
    let condition_kind = *bytes
        .get(start + 6)
        .ok_or_else(|| format!("command at 0x{start:x} has no condition kind"))?;
    let size_pos = match condition_kind {
        0 => start + 11,
        1 => start + 7,
        2 => start + 15,
        3 => {
            let expression_size = read_u16(bytes, start + 7, "condition expression size")? as usize;
            let cache_pos = checked_add(start + 9, expression_size, "condition cache")?;
            let compiled_pos = cache_pos
                .checked_add(1)
                .ok_or("condition compiled block offset overflow")?;
            let compiled_size =
                read_u16(bytes, compiled_pos, "condition compiled block size")? as usize;
            if compiled_size < 2 {
                return Err(format!(
                    "command at 0x{start:x} has a compiled condition size below 2"
                ));
            }
            checked_add(compiled_pos, compiled_size, "command size field")?
        }
        other => {
            return Err(format!(
                "command at 0x{start:x} has unknown condition kind {other}"
            ))
        }
    };
    let command_size = read_u32(bytes, size_pos, "command size")? as usize;
    if command_size < 5 {
        return Err(format!("command at 0x{start:x} has a size below 5"));
    }
    let end = checked_add(size_pos, command_size, "command end")?;
    if end > bytes.len() {
        return Err(format!("command at 0x{start:x} exceeds the file"));
    }
    let count_pos = size_pos + 4;
    let parameter_count = *bytes
        .get(count_pos)
        .ok_or_else(|| format!("command at 0x{start:x} has no parameter count"))?
        as usize;
    let mut cursor = count_pos + 1;
    let mut parameters = Vec::with_capacity(parameter_count);
    for index in 0..parameter_count {
        let parameter = parse_parameter(bytes, cursor, end)
            .map_err(|error| format!("command at 0x{start:x}, parameter {index}: {error}"))?;
        cursor = parameter.end;
        parameters.push(parameter);
    }
    if cursor != end {
        return Err(format!(
            "command at 0x{start:x} parameters end at 0x{cursor:x}, expected 0x{end:x}"
        ));
    }
    Ok(Command {
        start,
        end,
        opcode,
        condition_kind,
        size_pos,
        parameters,
    })
}

fn parse_parameter(bytes: &[u8], start: usize, command_end: usize) -> Result<Parameter> {
    let outer_kind = *bytes
        .get(start)
        .ok_or_else(|| "parameter kind is truncated".to_string())?;
    let mut result = Parameter {
        start,
        end: start + 1,
        outer_kind,
        inner_kind: None,
        kind: ParameterKind::Null,
        value_pos: None,
        length_pos: None,
        string: None,
    };
    match outer_kind {
        0 => {}
        2 => {
            result.kind = ParameterKind::Immediate;
            result.value_pos = Some(start + 1);
            result.end = start + 5;
        }
        _ => {
            let inner_kind = *bytes
                .get(start + 1)
                .ok_or_else(|| "parameter value kind is truncated".to_string())?;
            result.inner_kind = Some(inner_kind);
            match inner_kind {
                0 => {
                    result.kind = ParameterKind::Dword;
                    result.value_pos = Some(start + 2);
                    result.end = start + 6;
                }
                1 => {
                    let length_pos = start + 2;
                    let length = read_u16(bytes, length_pos, "string parameter size")? as usize;
                    let value_start = start + 4;
                    let value_end = checked_add(value_start, length, "string parameter end")?;
                    if length != 0 && bytes.get(value_end - 1) != Some(&0) {
                        return Err("string parameter is not NUL-terminated".to_string());
                    }
                    result.kind = ParameterKind::String;
                    result.length_pos = Some(length_pos);
                    result.string = Some(value_start..value_end.saturating_sub(1));
                    result.end = value_end;
                }
                2 => {
                    result.kind = ParameterKind::TableValue;
                    result.value_pos = Some(start + 2);
                    result.end = start + 10;
                }
                3 => {
                    let expression_size =
                        read_u16(bytes, start + 2, "parameter expression size")? as usize;
                    let cache_pos =
                        checked_add(start + 4, expression_size, "parameter expression cache")?;
                    let compiled_pos = cache_pos
                        .checked_add(1)
                        .ok_or("parameter compiled block offset overflow")?;
                    let compiled_size =
                        read_u16(bytes, compiled_pos, "parameter compiled block size")? as usize;
                    if compiled_size < 2 {
                        return Err("parameter compiled block size is below 2".to_string());
                    }
                    result.kind = ParameterKind::Expression;
                    result.end = checked_add(compiled_pos, compiled_size, "parameter end")?;
                }
                other => return Err(format!("unknown parameter value kind {other}")),
            }
        }
    }
    if result.end > command_end {
        return Err(format!(
            "parameter ends at 0x{:x}, beyond command end 0x{command_end:x}",
            result.end
        ));
    }
    Ok(result)
}

fn validate_targets(file: &TobFile) -> Result<()> {
    for (index, target) in file.offsets.iter().enumerate() {
        file.command_at_code_offset(*target).ok_or_else(|| {
            format!("offset table target {index} (0x{target:x}) is not a command boundary")
        })?;
    }
    for (index, label) in file.labels.iter().enumerate() {
        if label.target != u32::MAX && !is_code_boundary(file, label.target) {
            return Err(format!(
                "label {index} target 0x{:x} is not a code boundary",
                label.target
            ));
        }
    }
    for command in file.regions.iter().filter_map(|region| match region {
        CodeRegion::Command(value) => Some(value),
        _ => None,
    }) {
        if command.opcode > 3 {
            continue;
        }
        let first = command.parameters.first().ok_or_else(|| {
            format!(
                "jump opcode {} at 0x{:x} has no target",
                command.opcode, command.start
            )
        })?;
        if !matches!(first.kind, ParameterKind::Immediate | ParameterKind::Dword) {
            return Err(format!(
                "jump opcode {} at 0x{:x} has a non-static target",
                command.opcode, command.start
            ));
        }
        let target = read_u32(
            &file.bytes,
            first.value_pos.expect("immediate has a value position"),
            "jump target",
        )?;
        if target != u32::MAX && target as usize >= file.code_len() {
            return Err(format!(
                "jump opcode {} at 0x{:x} targets 0x{target:x} outside the code",
                command.opcode, command.start
            ));
        }
        if target != u32::MAX && !is_code_boundary(file, target) {
            return Err(format!(
                "jump opcode {} at 0x{:x} targets 0x{target:x}, which is not a code boundary",
                command.opcode, command.start
            ));
        }
    }
    Ok(())
}

fn is_code_boundary(file: &TobFile, target: u32) -> bool {
    let absolute = file.code_start + target as usize;
    absolute == file.code_start
        || absolute == file.opaque_tail.start
        || file.regions.iter().any(|region| region.start() == absolute)
}

fn read_u16(bytes: &[u8], position: usize, field: &str) -> Result<u16> {
    let raw = bytes
        .get(position..position + 2)
        .ok_or_else(|| format!("{field} at 0x{position:x} is truncated"))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

pub fn read_u32(bytes: &[u8], position: usize, field: &str) -> Result<u32> {
    let raw = bytes
        .get(position..position + 4)
        .ok_or_else(|| format!("{field} at 0x{position:x} is truncated"))?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn checked_add(base: usize, amount: usize, field: &str) -> Result<usize> {
    base.checked_add(amount)
        .ok_or_else(|| format!("{field} offset overflow"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_observed_condition_one_command_layout() {
        let bytes = [
            b'[', b' ', 0x12, 0, 0, 0, 1, 13, 0, 0, 0, 1, 1, 1, 4, 0, b'A', b'B', b'C', 0,
        ];
        let command = parse_command(&bytes, 0).expect("command parses");
        assert_eq!(command.opcode, 0x12);
        assert_eq!(command.end, bytes.len());
        assert_eq!(command.parameters.len(), 1);
        assert_eq!(command.parameters[0].string, Some(16..19));
    }
}
