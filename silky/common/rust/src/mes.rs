use crate::codec::{
    encode_text, is_multibyte_lead, is_shift_jis, is_utf8, try_decode_text, utf8_byte_count,
};
use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct CommandSpec {
    pub opcode: u8,
    pub args: &'static str,
    pub name: &'static str,
}

pub const COMMAND_LIBRARY: &[CommandSpec] = &[
    CommandSpec {
        opcode: 0x00,
        args: "",
        name: "RETURN",
    },
    CommandSpec {
        opcode: 0x01,
        args: "I",
        name: "",
    },
    CommandSpec {
        opcode: 0x02,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x03,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x04,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x05,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x06,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x0a,
        args: "S",
        name: "STR_CRYPT",
    },
    CommandSpec {
        opcode: 0x0b,
        args: "S",
        name: "STR_UNCRYPT",
    },
    CommandSpec {
        opcode: 0x0c,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x0d,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x0e,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x0f,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x10,
        args: "B",
        name: "",
    },
    CommandSpec {
        opcode: 0x11,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x14,
        args: ">I",
        name: "JUMP",
    },
    CommandSpec {
        opcode: 0x15,
        args: ">I",
        name: "MSG_OFSETTER",
    },
    CommandSpec {
        opcode: 0x16,
        args: ">I",
        name: "SPEC_OFSETTER",
    },
    CommandSpec {
        opcode: 0x17,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x18,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x19,
        args: ">I",
        name: "MESSAGE",
    },
    CommandSpec {
        opcode: 0x1a,
        args: ">I",
        name: "",
    },
    CommandSpec {
        opcode: 0x1b,
        args: ">I",
        name: "",
    },
    CommandSpec {
        opcode: 0x1c,
        args: "B",
        name: "TO_NEW_STRING",
    },
    CommandSpec {
        opcode: 0x32,
        args: "i",
        name: "PUSH",
    },
    CommandSpec {
        opcode: 0x33,
        args: "S",
        name: "PUSH_STR",
    },
    CommandSpec {
        opcode: 0x34,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x35,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x36,
        args: "B",
        name: "JUMP_2",
    },
    CommandSpec {
        opcode: 0x37,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x38,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x3a,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x3b,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x3c,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x3d,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x3e,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x3f,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x40,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x41,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x42,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0x43,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0xfa,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0xfb,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0xfc,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0xfd,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0xfe,
        args: "",
        name: "",
    },
    CommandSpec {
        opcode: 0xff,
        args: "",
        name: "",
    },
];

pub fn command_spec(opcode: u8) -> Option<&'static CommandSpec> {
    COMMAND_LIBRARY.iter().find(|spec| spec.opcode == opcode)
}

fn has_offset_operand(opcode: u8) -> bool {
    matches!(opcode, 0x14 | 0x15 | 0x16 | 0x1b)
}

#[derive(Clone, Debug)]
pub struct EncodedString {
    raw: Vec<u8>,
    decoded: Option<String>,
    replacement: Option<String>,
}

impl EncodedString {
    pub fn text(&self) -> Option<&str> {
        self.replacement.as_deref().or(self.decoded.as_deref())
    }

    pub fn original_text(&self) -> Option<&str> {
        self.decoded.as_deref()
    }

    pub fn raw_len(&self) -> usize {
        self.raw.len()
    }

    pub fn set_text(&mut self, value: String) {
        self.replacement = Some(value);
    }

    fn encoded(&self, opcode: u8, encoding: &str) -> Result<Vec<u8>> {
        match &self.replacement {
            Some(text) => encode_script_string(opcode, text, encoding),
            None => Ok(self.raw.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Argument {
    Integer(i64),
    String(EncodedString),
}

impl Argument {
    pub fn integer(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            Self::String(_) => None,
        }
    }

    pub fn string(&self) -> Option<&EncodedString> {
        match self {
            Self::String(value) => Some(value),
            Self::Integer(_) => None,
        }
    }

    pub fn string_mut(&mut self) -> Option<&mut EncodedString> {
        match self {
            Self::String(value) => Some(value),
            Self::Integer(_) => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Command {
    pub opcode: u8,
    pub args: Vec<Argument>,
}

impl Command {
    pub fn name(&self) -> &'static str {
        command_spec(self.opcode).map_or("", |spec| spec.name)
    }

    pub fn first_integer(&self) -> Option<i64> {
        self.args.first().and_then(Argument::integer)
    }

    pub fn first_string(&self) -> Option<&EncodedString> {
        self.args.first().and_then(Argument::string)
    }

    pub fn first_string_mut(&mut self) -> Option<&mut EncodedString> {
        self.args.first_mut().and_then(Argument::string_mut)
    }
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    Command(Command),
    Raw(u8),
}

#[derive(Clone, Debug)]
pub struct Node {
    pub old_offset: u32,
    pub old_len: u32,
    pub kind: NodeKind,
}

#[derive(Clone, Debug)]
pub struct MesScript {
    pub first_offsets: Vec<u32>,
    pub second_offsets: Vec<u32>,
    pub nodes: Vec<Node>,
    pub encoding: String,
    pub warnings: Vec<String>,
}

impl MesScript {
    pub fn parse(data: &[u8], encoding: &str) -> Result<Self> {
        if data.len() < 8 {
            bail!("MES file is smaller than its 8-byte header");
        }
        let message_count = read_u32_le(data, 0)? as usize;
        let special_count = read_u32_le(data, 4)? as usize;
        let table_words = message_count
            .checked_add(special_count)
            .ok_or_else(|| anyhow!("MES header count overflow"))?;
        let header_size = 8usize
            .checked_add(
                table_words
                    .checked_mul(4)
                    .ok_or_else(|| anyhow!("MES header size overflow"))?,
            )
            .ok_or_else(|| anyhow!("MES header size overflow"))?;
        if header_size > data.len() {
            bail!(
                "MES header exceeds file: header={header_size}, file={}",
                data.len()
            );
        }

        let mut pos = 8usize;
        let mut first_offsets = Vec::with_capacity(message_count);
        for _ in 0..message_count {
            first_offsets.push(read_u32_le(data, pos)?);
            pos += 4;
        }
        let mut second_offsets = Vec::with_capacity(special_count);
        for _ in 0..special_count {
            second_offsets.push(read_u32_le(data, pos)?);
            pos += 4;
        }

        let code = &data[header_size..];
        let code_len = u32::try_from(code.len()).context("MES code exceeds 4 GiB")?;
        for (kind, offsets) in [("message", &first_offsets), ("special", &second_offsets)] {
            for (index, offset) in offsets.iter().enumerate() {
                if *offset >= code_len {
                    bail!(
                        "MES {kind} offset out of bounds: index={index}, offset={offset}, code={code_len}"
                    );
                }
            }
        }

        let mut nodes = Vec::new();
        let mut warnings = Vec::new();
        let mut code_pos = 0usize;
        while code_pos < code.len() {
            let start = code_pos;
            let opcode = code[code_pos];
            code_pos += 1;
            let Some(spec) = command_spec(opcode) else {
                nodes.push(Node {
                    old_offset: start as u32,
                    old_len: 1,
                    kind: NodeKind::Raw(opcode),
                });
                continue;
            };

            let args = parse_arguments(code, &mut code_pos, spec, encoding).with_context(|| {
                format!("MES command 0x{opcode:02X} at code offset 0x{start:08X}")
            })?;
            for arg in &args {
                if let Argument::String(value) = arg {
                    if value.decoded.is_none() {
                        warnings.push(format!(
                            "undecodable string preserved at code offset 0x{start:08X}, opcode=0x{opcode:02X}"
                        ));
                    }
                }
            }
            let old_len = u32::try_from(code_pos - start).context("MES command too large")?;
            nodes.push(Node {
                old_offset: start as u32,
                old_len,
                kind: NodeKind::Command(Command { opcode, args }),
            });
        }

        let script = Self {
            first_offsets,
            second_offsets,
            nodes,
            encoding: encoding.to_owned(),
            warnings,
        };
        script.validate_targets(code_len)?;
        Ok(script)
    }

    pub fn header_size(&self) -> usize {
        8 + (self.first_offsets.len() + self.second_offsets.len()) * 4
    }

    pub fn command_node_indices(&self) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| match node.kind {
                NodeKind::Command(_) => Some(index),
                NodeKind::Raw(_) => None,
            })
            .collect()
    }

    pub fn command(&self, node_index: usize) -> Option<&Command> {
        match &self.nodes.get(node_index)?.kind {
            NodeKind::Command(command) => Some(command),
            NodeKind::Raw(_) => None,
        }
    }

    pub fn command_mut(&mut self, node_index: usize) -> Option<&mut Command> {
        match &mut self.nodes.get_mut(node_index)?.kind {
            NodeKind::Command(command) => Some(command),
            NodeKind::Raw(_) => None,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut relocation = HashMap::new();
        let mut new_code_offset = 0u32;
        for node in &self.nodes {
            relocation.insert(node.old_offset, new_code_offset);
            let len = match &node.kind {
                NodeKind::Raw(_) => 1usize,
                NodeKind::Command(command) => command_bytes(command, &self.encoding, None)?.len(),
            };
            new_code_offset = new_code_offset
                .checked_add(u32::try_from(len).context("rebuilt MES command too large")?)
                .ok_or_else(|| anyhow!("rebuilt MES code exceeds 4 GiB"))?;
        }
        let old_code_end = self
            .nodes
            .last()
            .map_or(0, |node| node.old_offset + node.old_len);
        relocation.insert(old_code_end, new_code_offset);

        let mut output = Vec::new();
        output.extend_from_slice(
            &u32::try_from(self.first_offsets.len())
                .context("too many MES message offsets")?
                .to_le_bytes(),
        );
        output.extend_from_slice(
            &u32::try_from(self.second_offsets.len())
                .context("too many MES special offsets")?
                .to_le_bytes(),
        );
        for offset in self.first_offsets.iter().chain(&self.second_offsets) {
            let relocated = relocation.get(offset).ok_or_else(|| {
                anyhow!("MES header offset does not target a structure boundary: 0x{offset:08X}")
            })?;
            output.extend_from_slice(&relocated.to_le_bytes());
        }

        for node in &self.nodes {
            match &node.kind {
                NodeKind::Raw(byte) => output.push(*byte),
                NodeKind::Command(command) => {
                    output.extend_from_slice(&command_bytes(
                        command,
                        &self.encoding,
                        Some(&relocation),
                    )?);
                }
            }
        }
        Ok(output)
    }

    fn validate_targets(&self, code_len: u32) -> Result<()> {
        let boundaries: HashMap<u32, ()> = self
            .nodes
            .iter()
            .map(|node| (node.old_offset, ()))
            .chain(std::iter::once((code_len, ())))
            .collect();
        for offset in self.first_offsets.iter().chain(&self.second_offsets) {
            if !boundaries.contains_key(offset) {
                bail!("MES header offset is not on an instruction/data boundary: 0x{offset:08X}");
            }
        }
        for node in &self.nodes {
            let NodeKind::Command(command) = &node.kind else {
                continue;
            };
            if has_offset_operand(command.opcode) {
                let value = command
                    .first_integer()
                    .ok_or_else(|| anyhow!("offset command has no integer operand"))?;
                let target = u32::try_from(value).with_context(|| {
                    format!(
                        "negative/oversized jump target at 0x{:08X}",
                        node.old_offset
                    )
                })?;
                if !boundaries.contains_key(&target) {
                    bail!(
                        "jump target is not on an instruction/data boundary: command=0x{:08X}, target=0x{target:08X}",
                        node.old_offset
                    );
                }
            }
        }
        Ok(())
    }
}

fn parse_arguments(
    code: &[u8],
    pos: &mut usize,
    spec: &CommandSpec,
    encoding: &str,
) -> Result<Vec<Argument>> {
    let mut args = Vec::new();
    let mut big_endian = false;
    for kind in spec.args.chars() {
        if kind == '>' {
            big_endian = true;
            continue;
        }
        match kind {
            'I' => {
                let bytes = take::<4>(code, pos)?;
                let value = if big_endian {
                    u32::from_be_bytes(bytes)
                } else {
                    u32::from_le_bytes(bytes)
                };
                args.push(Argument::Integer(value as i64));
            }
            'i' => {
                let bytes = take::<4>(code, pos)?;
                let value = if big_endian {
                    i32::from_be_bytes(bytes)
                } else {
                    i32::from_le_bytes(bytes)
                };
                args.push(Argument::Integer(value as i64));
            }
            'B' => {
                let byte = *code
                    .get(*pos)
                    .ok_or_else(|| anyhow!("truncated u8 operand"))?;
                *pos += 1;
                args.push(Argument::Integer(byte as i64));
            }
            'S' => {
                let relative_end = code[*pos..]
                    .iter()
                    .position(|byte| *byte == 0)
                    .ok_or_else(|| anyhow!("unterminated string operand"))?;
                let end = *pos + relative_end;
                let raw = code[*pos..end].to_vec();
                *pos = end + 1;
                let decoded = decode_script_string(spec.opcode, &raw, encoding)?;
                args.push(Argument::String(EncodedString {
                    raw,
                    decoded,
                    replacement: None,
                }));
            }
            other => bail!("unsupported MES operand schema: {other}"),
        }
        big_endian = false;
    }
    Ok(args)
}

fn command_bytes(
    command: &Command,
    encoding: &str,
    relocation: Option<&HashMap<u32, u32>>,
) -> Result<Vec<u8>> {
    let spec = command_spec(command.opcode)
        .ok_or_else(|| anyhow!("unknown MES opcode 0x{:02X}", command.opcode))?;
    let expected_args = spec
        .args
        .chars()
        .filter(|kind| !matches!(kind, '>' | '<'))
        .count();
    if command.args.len() != expected_args {
        bail!(
            "opcode 0x{:02X} has {} arguments; expected {expected_args}",
            command.opcode,
            command.args.len()
        );
    }

    let mut output = vec![command.opcode];
    let mut big_endian = false;
    let mut argument_index = 0usize;
    for kind in spec.args.chars() {
        if kind == '>' {
            big_endian = true;
            continue;
        }
        let argument = &command.args[argument_index];
        match kind {
            'I' => {
                let mut value = u32::try_from(
                    argument
                        .integer()
                        .ok_or_else(|| anyhow!("expected integer operand"))?,
                )?;
                if argument_index == 0 && has_offset_operand(command.opcode) {
                    if let Some(map) = relocation {
                        value = *map.get(&value).ok_or_else(|| {
                            anyhow!("jump target does not have a relocation: 0x{value:08X}")
                        })?;
                    }
                }
                if big_endian {
                    output.extend_from_slice(&value.to_be_bytes());
                } else {
                    output.extend_from_slice(&value.to_le_bytes());
                }
            }
            'i' => {
                let value = i32::try_from(
                    argument
                        .integer()
                        .ok_or_else(|| anyhow!("expected signed integer operand"))?,
                )?;
                if big_endian {
                    output.extend_from_slice(&value.to_be_bytes());
                } else {
                    output.extend_from_slice(&value.to_le_bytes());
                }
            }
            'B' => output.push(u8::try_from(
                argument
                    .integer()
                    .ok_or_else(|| anyhow!("expected u8 operand"))?,
            )?),
            'S' => {
                let value = argument
                    .string()
                    .ok_or_else(|| anyhow!("expected string operand"))?;
                output.extend_from_slice(&value.encoded(command.opcode, encoding)?);
                output.push(0);
            }
            other => bail!("unsupported MES operand schema: {other}"),
        }
        argument_index += 1;
        big_endian = false;
    }
    Ok(output)
}

fn decode_script_string(opcode: u8, raw: &[u8], encoding: &str) -> Result<Option<String>> {
    let decoded_bytes = if opcode == 0x0a {
        let mut output = Vec::new();
        let mut index = 0usize;
        while index < raw.len() {
            let byte = raw[index];
            if !is_multibyte_lead(byte, encoding) {
                let value = ((byte as i32 - 0x7d62) & 0xffff) as u16;
                output.push((value >> 8) as u8);
                output.push(value as u8);
                index += 1;
            } else if is_utf8(encoding) {
                let length = utf8_byte_count(byte).min(raw.len() - index);
                output.extend_from_slice(&raw[index..index + length]);
                index += length;
            } else {
                output.push(byte);
                index += 1;
                if index < raw.len() {
                    output.push(raw[index]);
                    index += 1;
                }
            }
        }
        output
    } else if matches!(opcode, 0x0b | 0x33) {
        raw.to_vec()
    } else {
        return Ok(None);
    };
    try_decode_text(&decoded_bytes, encoding)
}

pub(crate) fn encode_script_string(opcode: u8, text: &str, encoding: &str) -> Result<Vec<u8>> {
    let raw = encode_text(text, encoding)?;
    if opcode != 0x0a {
        return Ok(raw);
    }

    let mut output = Vec::with_capacity(raw.len());
    let mut index = 0usize;
    while index < raw.len() {
        let byte = raw[index];
        if is_utf8(encoding) && byte >= 0xc0 {
            let length = utf8_byte_count(byte).min(raw.len() - index);
            output.extend_from_slice(&raw[index..index + length]);
            index += length;
            continue;
        }
        if is_multibyte_lead(byte, encoding) && index + 1 < raw.len() {
            let value = u16::from_be_bytes([raw[index], raw[index + 1]]);
            if is_shift_jis(encoding) && (0x829f..=0x831e).contains(&value) {
                let compressed = value - 0x829e;
                if compressed < 0x81 {
                    output.push(compressed as u8);
                    index += 2;
                    continue;
                }
            }
            output.extend_from_slice(&raw[index..index + 2]);
            index += 2;
            continue;
        }
        output.push(byte);
        index += 1;
    }
    Ok(output)
}

fn take<const N: usize>(data: &[u8], pos: &mut usize) -> Result<[u8; N]> {
    let end = pos
        .checked_add(N)
        .ok_or_else(|| anyhow!("operand offset overflow"))?;
    let slice = data
        .get(*pos..end)
        .ok_or_else(|| anyhow!("truncated {N}-byte operand"))?;
    *pos = end;
    Ok(slice.try_into().expect("slice length checked"))
}

fn read_u32_le(data: &[u8], pos: usize) -> Result<u32> {
    let bytes = data
        .get(pos..pos + 4)
        .ok_or_else(|| anyhow!("truncated u32 at file offset 0x{pos:08X}"))?;
    Ok(u32::from_le_bytes(
        bytes.try_into().expect("slice length checked"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<u8> {
        let name = encode_text("リルカ", "cp932").unwrap();
        let body = encode_script_string(0x0a, "あいう", "cp932").unwrap();
        let mut code = Vec::new();
        code.push(0x19);
        code.extend_from_slice(&0u32.to_be_bytes());
        code.push(0x33);
        code.extend_from_slice(&name);
        code.push(0);
        code.push(0x32);
        code.extend_from_slice(&117_440_512i32.to_le_bytes());
        code.push(0x32);
        code.extend_from_slice(&486_539_264i32.to_le_bytes());
        code.push(0x18);
        code.push(0x0a);
        code.extend_from_slice(&body);
        code.push(0);
        let return_offset = code.len() as u32;
        code.push(0x00);
        code.push(0x14);
        code.extend_from_slice(&return_offset.to_be_bytes());

        let mut file = Vec::new();
        file.extend_from_slice(&1u32.to_le_bytes());
        file.extend_from_slice(&1u32.to_le_bytes());
        file.extend_from_slice(&0u32.to_le_bytes());
        file.extend_from_slice(&return_offset.to_le_bytes());
        file.extend_from_slice(&code);
        file
    }

    #[test]
    fn unchanged_parse_rebuild_is_byte_exact() {
        let source = fixture();
        let script = MesScript::parse(&source, "cp932").unwrap();
        assert_eq!(script.to_bytes().unwrap(), source);
        assert!(script.warnings.is_empty());
    }

    #[test]
    fn changed_string_relocates_header_and_jump() {
        let source = fixture();
        let mut script = MesScript::parse(&source, "cp932").unwrap();
        let body_node = script
            .nodes
            .iter()
            .position(
                |node| matches!(&node.kind, NodeKind::Command(command) if command.opcode == 0x0a),
            )
            .unwrap();
        script
            .command_mut(body_node)
            .unwrap()
            .first_string_mut()
            .unwrap()
            .set_text("あいうえおかきくけこ".to_owned());
        let rebuilt = script.to_bytes().unwrap();
        let reparsed = MesScript::parse(&rebuilt, "cp932").unwrap();
        assert_eq!(
            reparsed.second_offsets[0] as usize + reparsed.header_size(),
            rebuilt.len() - 6
        );
        assert_eq!(
            reparsed
                .nodes
                .iter()
                .find_map(|node| match &node.kind {
                    NodeKind::Command(command) if command.opcode == 0x14 => command.first_integer(),
                    _ => None,
                })
                .unwrap() as u32,
            reparsed.second_offsets[0]
        );
    }
}
