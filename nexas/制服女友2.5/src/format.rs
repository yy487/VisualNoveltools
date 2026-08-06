use crate::{ToolResult, error};

const HEADER: &[u8; 9] = b"VER-1.00\0";
const MAX_COUNT: usize = 1_000_000;

#[derive(Clone, Debug)]
pub struct StringSlot {
    pub value: String,
    pub original: String,
    pub raw: Vec<u8>,
    pub offset: usize,
    pub valid_utf8: bool,
}

impl StringSlot {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            original: value.clone(),
            raw: value.as_bytes().to_vec(),
            value,
            offset: 0,
            valid_utf8: true,
        }
    }

    fn write_into(&self, out: &mut Vec<u8>) {
        if self.value == self.original {
            out.extend_from_slice(&self.raw);
        } else {
            out.extend_from_slice(self.value.as_bytes());
        }
        out.push(0);
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub id: i32,
    pub unknown_array1: Vec<[i32; 2]>,
    pub opcodes: Vec<[i32; 2]>,
    pub constants: Vec<StringSlot>,
    pub local_variables: Vec<StringSlot>,
    pub parameters: Vec<StringSlot>,
    pub unknown_blocks: Vec<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct Script {
    pub unknown_array1: Vec<i32>,
    pub unknown_array2: Vec<[i32; 2]>,
    pub opcodes: Vec<[i32; 2]>,
    pub constants: Vec<StringSlot>,
    pub variables: Vec<StringSlot>,
    pub parameters: Vec<StringSlot>,
    pub unknown_blocks: Vec<Vec<u8>>,
    pub functions: Vec<Function>,
}

impl Script {
    pub fn parse(data: &[u8], source: &str) -> ToolResult<Self> {
        let mut reader = Reader::new(data, source);
        let header = reader.take(9)?;
        if header != HEADER {
            return Err(error(format!(
                "{source}: invalid NeXAS script header (expected VER-1.00\\0)"
            )));
        }

        let unknown_array1 = reader.read_i32_vec("global unknown array #1")?;
        let unknown_array2 = reader.read_pair_vec("global unknown array #2")?;
        let opcodes = reader.read_pair_vec("global opcode table")?;
        let constants = reader.read_string_vec("global constant string table")?;
        let variables = reader.read_string_vec("global variable declarations")?;
        let parameters = reader.read_string_vec("global parameter declarations")?;
        let unknown_blocks = reader.read_block_vec("global opaque block table", 68)?;

        let mut functions = Vec::new();
        while reader.remaining() > 4 {
            let id = reader.read_i32("function id")?;
            let unknown_array1 = reader.read_pair_vec("function unknown array #1")?;
            let opcodes = reader.read_pair_vec("function opcode table")?;
            let constants = reader.read_string_vec("function constant string table")?;
            let local_variables = reader.read_string_vec("function local declarations")?;
            let parameters = reader.read_string_vec("function parameter declarations")?;
            let unknown_blocks = reader.read_block_vec("function opaque block table", 68)?;
            functions.push(Function {
                id,
                unknown_array1,
                opcodes,
                constants,
                local_variables,
                parameters,
                unknown_blocks,
            });
        }

        if reader.remaining() != 0 {
            return Err(error(format!(
                "{source}: {} trailing bytes after function table",
                reader.remaining()
            )));
        }

        Ok(Self {
            unknown_array1,
            unknown_array2,
            opcodes,
            constants,
            variables,
            parameters,
            unknown_blocks,
            functions,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(HEADER);
        write_i32_vec(&mut out, &self.unknown_array1);
        write_pair_vec(&mut out, &self.unknown_array2);
        write_pair_vec(&mut out, &self.opcodes);
        write_string_vec(&mut out, &self.constants);
        write_string_vec(&mut out, &self.variables);
        write_string_vec(&mut out, &self.parameters);
        write_block_vec(&mut out, &self.unknown_blocks);

        for function in &self.functions {
            write_i32(&mut out, function.id);
            write_pair_vec(&mut out, &function.unknown_array1);
            write_pair_vec(&mut out, &function.opcodes);
            write_string_vec(&mut out, &function.constants);
            write_string_vec(&mut out, &function.local_variables);
            write_string_vec(&mut out, &function.parameters);
            write_block_vec(&mut out, &function.unknown_blocks);
        }
        out
    }

    pub fn string_count(&self) -> usize {
        self.constants.len()
            + self.variables.len()
            + self.parameters.len()
            + self
                .functions
                .iter()
                .map(|function| {
                    function.constants.len()
                        + function.local_variables.len()
                        + function.parameters.len()
                })
                .sum::<usize>()
    }
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    source: &'a str,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8], source: &'a str) -> Self {
        Self {
            data,
            pos: 0,
            source,
        }
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn take(&mut self, size: usize) -> ToolResult<&'a [u8]> {
        let end = self
            .pos
            .checked_add(size)
            .ok_or_else(|| error(format!("{}: offset overflow", self.source)))?;
        if end > self.data.len() {
            return Err(error(format!(
                "{}: truncated at offset {}, need {} bytes",
                self.source, self.pos, size
            )));
        }
        let start = self.pos;
        self.pos = end;
        Ok(&self.data[start..end])
    }

    fn read_i32(&mut self, context: &str) -> ToolResult<i32> {
        let bytes = self.take(4)?;
        Ok(i32::from_le_bytes(bytes.try_into().map_err(|_| {
            error(format!("{}: invalid {context}", self.source))
        })?))
    }

    fn read_count(&mut self, context: &str) -> ToolResult<usize> {
        let value = self.read_i32(context)?;
        if value < 0 || value as usize > MAX_COUNT {
            return Err(error(format!(
                "{}: invalid {context} count {value} at offset {}",
                self.source,
                self.pos.saturating_sub(4)
            )));
        }
        Ok(value as usize)
    }

    fn read_i32_vec(&mut self, context: &str) -> ToolResult<Vec<i32>> {
        let count = self.read_count(context)?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(self.read_i32(context)?);
        }
        Ok(values)
    }

    fn read_pair_vec(&mut self, context: &str) -> ToolResult<Vec<[i32; 2]>> {
        let count = self.read_count(context)?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push([self.read_i32(context)?, self.read_i32(context)?]);
        }
        Ok(values)
    }

    fn read_string_vec(&mut self, context: &str) -> ToolResult<Vec<StringSlot>> {
        let count = self.read_count(context)?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(self.read_string(context)?);
        }
        Ok(values)
    }

    fn read_string(&mut self, context: &str) -> ToolResult<StringSlot> {
        let offset = self.pos;
        let rest = &self.data[self.pos..];
        let end = rest.iter().position(|byte| *byte == 0).ok_or_else(|| {
            error(format!(
                "{}: unterminated string in {context} at offset {offset}",
                self.source
            ))
        })?;
        let raw = self.take(end)?.to_vec();
        self.take(1)?;
        let (value, valid_utf8) = match String::from_utf8(raw.clone()) {
            Ok(value) => (value, true),
            Err(error) => (
                String::from_utf8_lossy(error.as_bytes()).into_owned(),
                false,
            ),
        };
        Ok(StringSlot {
            original: value.clone(),
            value,
            raw,
            offset,
            valid_utf8,
        })
    }

    fn read_block_vec(&mut self, context: &str, block_size: usize) -> ToolResult<Vec<Vec<u8>>> {
        let count = self.read_count(context)?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(self.take(block_size)?.to_vec());
        }
        Ok(values)
    }
}

fn write_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_i32_vec(out: &mut Vec<u8>, values: &[i32]) {
    write_i32(out, values.len() as i32);
    for value in values {
        write_i32(out, *value);
    }
}

fn write_pair_vec(out: &mut Vec<u8>, values: &[[i32; 2]]) {
    write_i32(out, values.len() as i32);
    for value in values {
        write_i32(out, value[0]);
        write_i32(out, value[1]);
    }
}

fn write_string_vec(out: &mut Vec<u8>, values: &[StringSlot]) {
    write_i32(out, values.len() as i32);
    for value in values {
        value.write_into(out);
    }
}

fn write_block_vec(out: &mut Vec<u8>, values: &[Vec<u8>]) {
    write_i32(out, values.len() as i32);
    for value in values {
        out.extend_from_slice(value);
    }
}
