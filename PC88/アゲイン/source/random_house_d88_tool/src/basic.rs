use crate::renderer::{
    LOAD_ADDRESS, NATIVE_ASCII_SENTINEL, PAGE_CLEAR_SENTINEL, STATE_END, STATE_START,
};
use anyhow::{bail, Context, Result};

const TOKEN_FOR: u8 = 0x82;
const TOKEN_NEXT: u8 = 0x83;
const TOKEN_DATA: u8 = 0x84;
const TOKEN_REM: u8 = 0x8F;
const TOKEN_CLEAR: u8 = 0x92;
const TOKEN_PRINT: u8 = 0x91;
const TOKEN_CLS: u8 = 0xCE;
const TOKEN_POKE: u8 = 0x98;
const TOKEN_TO: u8 = 0xDC;
const TOKEN_EQUALS: u8 = 0xF1;
const TOKEN_PLUS: u8 = 0xF3;
const TOKEN_MINUS: u8 = 0xF4;
const TOKEN_MULTIPLY: u8 = 0xF5;
const TOKEN_MID_PREFIX: [u8; 2] = [0xFF, 0x83];
const TOKEN_ASC_PREFIX: [u8; 2] = [0xFF, 0x95];
const TOKEN_POLL: [u8; 2] = [0xFF, 0xDF];
const COMPACT_ZERO: u8 = 0x11;
const COMPACT_ONE: u8 = 0x12;
const COMPACT_TWO: u8 = 0x13;
const INTEGER_U8: u8 = 0x0F;
const INTEGER_U16: u8 = 0x0C;
const EXTENDED_TOKEN_PREFIX: u8 = 0xFF;
const CLS_INSTRUMENTATION: [u8; 7] = [
    TOKEN_PRINT,
    b'"',
    PAGE_CLEAR_SENTINEL[0],
    PAGE_CLEAR_SENTINEL[1],
    b'"',
    b';',
    b':',
];
const NATIVE_ASCII_PREFIX: [u8; 5] = [
    b'"',
    NATIVE_ASCII_SENTINEL[0],
    NATIVE_ASCII_SENTINEL[1],
    b'"',
    b';',
];

const PRELOAD_CLEAR_LINE_NUMBER: u16 = 191;
const PAYLOAD_LINE_START: u16 = 192;
const LOADER_LINE_START: u16 = 202;
const INSTALL_LINE_NUMBER: u16 = 209;
const PAYLOAD_LINE_STEP: u16 = 10;
const PAYLOAD_LINE_NUMBERS: [u16; 8] = [192, 194, 196, 198, 201, 203, 205, 207];
const PROBE_PAYLOAD_LINE_START: u16 = 2_000;
const PAYLOAD_SEGMENT_BYTES: usize = 110;
const PAYLOAD_NIBBLE_BASE: u8 = b'A';
const PAYLOAD_VARIABLE: &[u8] = b"RZ$";
const PROBE_PAYLOAD_MARKER: &[u8] = b"RHDBCS1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicLine {
    pub number: u16,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct BasicProgram {
    pub lines: Vec<BasicLine>,
    trailing: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct BootstrapResult {
    pub bytes: Vec<u8>,
    pub payload_line_start: u16,
    pub program_bytes: usize,
    pub allocated_bytes: usize,
    pub comment_bytes_removed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicQuotedString {
    pub index: usize,
    pub line_index: usize,
    pub line_number: u16,
    /// Offset of the first payload byte inside `BasicLine::body`.
    pub body_offset: usize,
    /// Original file offset of the first payload byte.
    pub file_offset: usize,
    pub statement_offset: usize,
    pub bytes: Vec<u8>,
    pub terminated: bool,
}

impl BasicProgram {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut position = 0usize;
        let mut lines = Vec::new();
        let mut previous_number = None;
        loop {
            let next_pointer = read_u16(data, position)?;
            if next_pointer == 0 {
                let trailing = data
                    .get(position + 2..)
                    .context("BASIC final terminator is truncated")?
                    .to_vec();
                return Ok(Self { lines, trailing });
            }
            let next_position = usize::from(next_pointer) - 1;
            if next_position <= position + 4 || next_position > data.len() {
                bail!(
                    "invalid BASIC next pointer 0x{next_pointer:04X} at file offset 0x{position:X}"
                );
            }
            let number = read_u16(data, position + 2)?;
            if previous_number.is_some_and(|previous| number <= previous) {
                bail!("BASIC line numbers are not strictly increasing at {number}");
            }
            let body = data[position + 4..next_position].to_vec();
            if body.last() != Some(&0) {
                bail!("BASIC line {number} has no NUL terminator");
            }
            lines.push(BasicLine { number, body });
            previous_number = Some(number);
            position = next_position;
        }
    }

    pub fn byte_exact_rebuild(&self) -> Result<Vec<u8>> {
        self.serialize_with_trailing(&self.trailing)
    }

    /// Rebuild a BASIC file while dropping only zero allocation padding after
    /// the final program terminator. Any opaque nonzero tail remains intact.
    pub fn compact_sector_rebuild(&self) -> Result<Vec<u8>> {
        let mut output = self.compact_unpadded_rebuild()?;
        output.resize(output.len().div_ceil(256) * 256, 0);
        Ok(output)
    }

    pub fn compact_unpadded_rebuild(&self) -> Result<Vec<u8>> {
        let trailing_end = self
            .trailing
            .iter()
            .rposition(|byte| *byte != 0)
            .map_or(0, |index| index + 1);
        self.serialize_with_trailing(&self.trailing[..trailing_end])
    }

    /// Remove at most `wanted` bytes from the ends of executable-inert REM
    /// comments. REM tokens and line numbers remain present, so GOTO targets and
    /// fall-through behavior are unchanged.
    pub fn trim_rem_comment_bytes(&mut self, mut wanted: usize) -> usize {
        let mut removed = 0usize;
        for line in self.lines.iter_mut().rev() {
            if wanted == 0 {
                break;
            }
            let Some(content_end) = line
                .body
                .len()
                .checked_sub(1)
                .filter(|_| line.body.last() == Some(&0))
            else {
                continue;
            };
            let mut quoted = false;
            let mut rem_offset = None;
            let mut offset = 0usize;
            while offset < content_end {
                let byte = line.body[offset];
                if byte == b'"' {
                    quoted = !quoted;
                    offset += 1;
                } else if !quoted && byte == TOKEN_REM {
                    rem_offset = Some(offset);
                    break;
                } else if !quoted {
                    let payload_len = n88_token_payload_len(byte);
                    let Some(next) = offset
                        .checked_add(1 + payload_len)
                        .filter(|next| *next <= content_end)
                    else {
                        break;
                    };
                    offset = next;
                } else {
                    offset += 1;
                }
            }
            let Some(comment_start) = rem_offset.map(|offset| offset + 1) else {
                continue;
            };
            let take = wanted.min(content_end.saturating_sub(comment_start));
            if take == 0 {
                continue;
            }
            line.body.drain(content_end - take..content_end);
            wanted -= take;
            removed += take;
        }
        removed
    }

    /// Remove interpreter-ignored code spaces without touching quoted text,
    /// DATA fields, REM bodies, or opaque token payloads.
    pub fn trim_ignored_space_bytes(&mut self, mut wanted: usize) -> usize {
        let mut removed = 0usize;
        for line in self.lines.iter_mut().rev() {
            if wanted == 0 {
                break;
            }
            let Some(content_end) = basic_content_end(&line.body) else {
                continue;
            };
            let mut candidates = Vec::new();
            let mut quoted = false;
            let mut data_statement = false;
            let mut offset = 0usize;
            while offset < content_end {
                let byte = line.body[offset];
                if byte == b'"' {
                    quoted = !quoted;
                    offset += 1;
                    continue;
                }
                if quoted {
                    offset += 1;
                    continue;
                }
                if byte == TOKEN_REM {
                    break;
                }
                if byte == TOKEN_DATA {
                    data_statement = true;
                    offset += 1;
                    continue;
                }
                if byte == b':' {
                    data_statement = false;
                    offset += 1;
                    continue;
                }
                if byte == b' ' && !data_statement {
                    candidates.push(offset);
                    offset += 1;
                    continue;
                }
                let payload_len = n88_token_payload_len(byte);
                let Some(next) = offset
                    .checked_add(1 + payload_len)
                    .filter(|next| *next <= content_end)
                else {
                    break;
                };
                offset = next;
            }

            let take = wanted.min(candidates.len());
            for offset in candidates.into_iter().take(take).rev() {
                line.body.remove(offset);
            }
            wanted -= take;
            removed += take;
        }
        removed
    }

    pub fn quoted_strings(&self) -> Result<Vec<BasicQuotedString>> {
        let mut strings = Vec::new();
        let mut file_position = 0usize;
        for (line_index, line) in self.lines.iter().enumerate() {
            let content_end = basic_content_end(&line.body)
                .context("BASIC line lost its final NUL terminator")?;
            let content = &line.body[..content_end];
            let mut opening = None;
            let mut statement_offset = 0usize;
            for (offset, byte) in content.iter().copied().enumerate() {
                if byte == b'"' {
                    if let Some(start) = opening.take() {
                        strings.push(BasicQuotedString {
                            index: strings.len(),
                            line_index,
                            line_number: line.number,
                            body_offset: start,
                            file_offset: file_position + 4 + start,
                            statement_offset,
                            bytes: content[start..offset].to_vec(),
                            terminated: true,
                        });
                    } else {
                        opening = Some(offset + 1);
                    }
                } else if byte == b':' && opening.is_none() {
                    statement_offset = offset + 1;
                }
            }
            if let Some(start) = opening {
                strings.push(BasicQuotedString {
                    index: strings.len(),
                    line_index,
                    line_number: line.number,
                    body_offset: start,
                    file_offset: file_position + 4 + start,
                    statement_offset,
                    bytes: content[start..].to_vec(),
                    terminated: false,
                });
            }
            file_position += 4 + line.body.len();
        }
        Ok(strings)
    }

    pub fn replace_quoted_string(
        &mut self,
        line_index: usize,
        body_offset: usize,
        expected: &[u8],
        replacement: &[u8],
        terminated: bool,
    ) -> Result<()> {
        let line = self
            .lines
            .get_mut(line_index)
            .with_context(|| format!("BASIC line index {line_index} is out of bounds"))?;
        if body_offset == 0 || line.body.get(body_offset - 1) != Some(&b'"') {
            bail!(
                "BASIC line {} has no opening quote at body offset 0x{:X}",
                line.number,
                body_offset.saturating_sub(1)
            );
        }
        let end = body_offset
            .checked_add(expected.len())
            .context("quoted string replacement offset overflow")?;
        let actual = line
            .body
            .get(body_offset..end)
            .with_context(|| format!("BASIC line {} quoted payload is truncated", line.number))?;
        if actual != expected {
            bail!(
                "BASIC line {} quoted source bytes changed at body offset 0x{body_offset:X}",
                line.number
            );
        }
        if terminated {
            if line.body.get(end) != Some(&b'"') {
                bail!(
                    "BASIC line {} lost its closing quote at body offset 0x{end:X}",
                    line.number
                );
            }
        } else if line.body.get(end) != Some(&0) {
            bail!(
                "BASIC line {} unterminated literal no longer reaches the statement terminator",
                line.number
            );
        }
        line.body
            .splice(body_offset..end, replacement.iter().copied());
        if line.body.len() > 255 {
            bail!(
                "BASIC line {} grows to {} bytes, exceeding the 255-byte line limit",
                line.number,
                line.body.len()
            );
        }
        Ok(())
    }

    /// Emit the resident renderer's immediate-clear command immediately before
    /// every executable CLS statement. Quoted bytes, REM bodies, and token
    /// payloads are never interpreted as statements.
    pub fn instrument_cls(&mut self) -> Result<usize> {
        let mut patches = 0usize;
        for line in &mut self.lines {
            let content_end = basic_content_end(&line.body)
                .context("BASIC line lost its final NUL terminator")?;
            let mut cls_offsets = Vec::new();
            let mut quoted = false;
            let mut data_statement = false;
            let mut offset = 0usize;
            while offset < content_end {
                let byte = line.body[offset];
                if byte == b'"' {
                    quoted = !quoted;
                    offset += 1;
                    continue;
                }
                if quoted {
                    offset += 1;
                    continue;
                }
                if byte == TOKEN_REM {
                    break;
                }
                if data_statement {
                    if byte == b':' {
                        data_statement = false;
                    }
                    offset += 1;
                    continue;
                }
                if byte == TOKEN_DATA {
                    data_statement = true;
                    offset += 1;
                    continue;
                }
                if byte == TOKEN_CLS {
                    let mut prefix_end = offset;
                    while prefix_end > 0 && line.body[prefix_end - 1] == b' ' {
                        prefix_end -= 1;
                    }
                    let is_poll_argument = prefix_end >= TOKEN_POLL.len()
                        && line.body[prefix_end - TOKEN_POLL.len()..prefix_end] == TOKEN_POLL;
                    if !is_poll_argument {
                        cls_offsets.push(offset);
                    }
                    offset += 1;
                    continue;
                }
                let payload_len = n88_token_payload_len(byte);
                offset = offset
                    .checked_add(1 + payload_len)
                    .filter(|next| *next <= content_end)
                    .with_context(|| {
                        format!(
                            "BASIC line {} has a truncated token payload at body offset 0x{offset:X}",
                            line.number
                        )
                    })?;
            }

            let growth = cls_offsets
                .len()
                .checked_mul(CLS_INSTRUMENTATION.len())
                .context("CLS instrumentation size overflow")?;
            ensure_line_size(line.number, line.body.len() + growth)?;
            for offset in cls_offsets.into_iter().rev() {
                line.body
                    .splice(offset..offset, CLS_INSTRUMENTATION.iter().copied());
                patches += 1;
            }
        }
        Ok(patches)
    }

    /// Insert an immediate translated-layer clear before one profiled BASIC
    /// line. The expected prefix keeps this a structural patch rather than a
    /// line-number-only replacement.
    pub fn instrument_page_clear_before_line(
        &mut self,
        line_number: u16,
        expected_prefix: &[u8],
    ) -> Result<()> {
        let line = self
            .lines
            .iter_mut()
            .find(|line| line.number == line_number)
            .with_context(|| format!("BASIC line {line_number} was not found"))?;
        if !line.body.starts_with(expected_prefix) {
            bail!("BASIC line {line_number} does not match its profiled statement prefix");
        }
        ensure_line_size(line.number, line.body.len() + CLS_INSTRUMENTATION.len())?;
        line.body.splice(0..0, CLS_INSTRUMENTATION.iter().copied());
        Ok(())
    }

    /// Prefix each dynamic `PRINT A$` with a renderer command that sends
    /// exactly its next printable ASCII byte through N88's native path.
    pub fn instrument_native_ascii_echo(&mut self) -> Result<usize> {
        let mut patches = 0usize;
        for line in &mut self.lines {
            let content_end = basic_content_end(&line.body)
                .context("BASIC line lost its final NUL terminator")?;
            let mut insertions = Vec::new();
            let mut quoted = false;
            let mut data_statement = false;
            let mut offset = 0usize;
            while offset < content_end {
                let byte = line.body[offset];
                if byte == b'"' {
                    quoted = !quoted;
                    offset += 1;
                    continue;
                }
                if quoted {
                    offset += 1;
                    continue;
                }
                if byte == TOKEN_REM {
                    break;
                }
                if byte == b':' {
                    data_statement = false;
                    offset += 1;
                    continue;
                }
                if data_statement {
                    offset += 1;
                    continue;
                }
                if byte == TOKEN_DATA {
                    data_statement = true;
                    offset += 1;
                    continue;
                }
                if byte == TOKEN_PRINT {
                    let mut variable = offset + 1;
                    while variable < content_end && line.body[variable] == b' ' {
                        variable += 1;
                    }
                    if line.body.get(variable..variable + 2) == Some(b"A$") {
                        insertions.push(offset + 1);
                    }
                }
                let payload_len = n88_token_payload_len(byte);
                offset = offset
                    .checked_add(1 + payload_len)
                    .filter(|next| *next <= content_end)
                    .with_context(|| {
                        format!(
                            "BASIC line {} has a truncated token payload at body offset 0x{offset:X}",
                            line.number
                        )
                    })?;
            }

            let growth = insertions
                .len()
                .checked_mul(NATIVE_ASCII_PREFIX.len())
                .context("native ASCII instrumentation size overflow")?;
            ensure_line_size(line.number, line.body.len() + growth)?;
            for insertion in insertions.into_iter().rev() {
                line.body
                    .splice(insertion..insertion, NATIVE_ASCII_PREFIX.iter().copied());
                patches += 1;
            }
        }
        Ok(patches)
    }

    pub fn patch_memory_ceiling(&mut self) -> Result<()> {
        let mut matches = Vec::new();
        for (line_index, line) in self.lines.iter().enumerate() {
            let content_end = basic_content_end(&line.body)
                .context("BASIC line lost its final NUL terminator")?;
            let content = &line.body[..content_end];
            if content.first() != Some(&TOKEN_CLEAR) {
                continue;
            }
            for offset in 0..content.len().saturating_sub(2) {
                if content[offset..offset + 3] == [INTEGER_U16, 0xFF, 0xCE] {
                    matches.push((line_index, offset));
                }
            }
        }
        if matches.len() != 1 {
            bail!(
                "expected exactly one leading CLEAR memory ceiling, found {}",
                matches.len()
            );
        }
        let (line_index, offset) = matches[0];
        self.lines[line_index].body[offset + 1] = 0xFF;
        self.lines[line_index].body[offset + 2] = 0xBB;
        Ok(())
    }

    pub fn embedded_payload(&self) -> Result<Option<Vec<u8>>> {
        let mut output = Vec::new();
        for (segment_index, number) in PAYLOAD_LINE_NUMBERS.iter().copied().enumerate() {
            let Some(line) = self.lines.iter().find(|line| line.number == number) else {
                if segment_index == 0 {
                    return Ok(None);
                }
                break;
            };
            let prefix = [PAYLOAD_VARIABLE, &[TOKEN_EQUALS, b'"']].concat();
            if line.body.len() < prefix.len() + 2
                || !line.body.starts_with(&prefix)
                || !line.body.ends_with(&[b'"', 0])
            {
                bail!("resident payload line {number} has an invalid string envelope");
            }
            let encoded = &line.body[prefix.len()..line.body.len() - 2];
            if encoded.len() % 2 != 0 || encoded.iter().any(|value| !(b'A'..=b'P').contains(value))
            {
                bail!("resident payload line {number} has malformed printable nibble encoding");
            }
            output.extend(encoded.chunks_exact(2).map(|pair| {
                ((pair[0] - PAYLOAD_NIBBLE_BASE) << 4) | (pair[1] - PAYLOAD_NIBBLE_BASE)
            }));
        }
        Ok(Some(output))
    }

    pub fn install_bootstrap(
        &mut self,
        renderer: &[u8],
        max_program_bytes: Option<usize>,
    ) -> Result<BootstrapResult> {
        if renderer.is_empty() || renderer.len() > 0xFFFF {
            bail!("renderer payload size {} is unsupported", renderer.len());
        }
        let segments = renderer.chunks(PAYLOAD_SEGMENT_BYTES).collect::<Vec<_>>();
        if segments.len() > PAYLOAD_LINE_NUMBERS.len() {
            bail!("renderer requires too many BASIC loader segments");
        }
        let reserved_lines = PAYLOAD_LINE_NUMBERS
            .iter()
            .flat_map(|number| [*number, number + 1])
            .chain([PRELOAD_CLEAR_LINE_NUMBER, INSTALL_LINE_NUMBER]);
        if reserved_lines
            .into_iter()
            .any(|number| self.lines.iter().any(|line| line.number == number))
        {
            bail!("BASIC bootstrap line number is already occupied");
        }

        self.relocate_leading_clear()?;

        // The BASIC program itself lives behind the low-address ROM bank, so PEEK
        // cannot read bytes from REM text.  Put each printable segment in a BASIC
        // string and decode it through ASC(MID$()) instead.
        for (index, segment) in segments.iter().enumerate() {
            let payload_line_number = PAYLOAD_LINE_NUMBERS[index];
            let loader_line_number = payload_line_number + 1;
            let destination = LOAD_ADDRESS
                .checked_add(u16::try_from(index)? * u16::try_from(PAYLOAD_SEGMENT_BYTES)?)
                .context("renderer destination address overflow")?;

            let mut payload_body = PAYLOAD_VARIABLE.to_vec();
            payload_body.extend_from_slice(&[TOKEN_EQUALS, b'"']);
            for value in *segment {
                payload_body.push((value >> 4) + PAYLOAD_NIBBLE_BASE);
                payload_body.push((value & 0x0F) + PAYLOAD_NIBBLE_BASE);
            }
            payload_body.extend_from_slice(&[b'"', 0]);
            self.lines.push(BasicLine {
                number: payload_line_number,
                body: payload_body,
            });
            self.lines.push(BasicLine {
                number: loader_line_number,
                body: decode_loader_body(segment.len(), destination)?,
            });
        }
        self.lines.push(BasicLine {
            number: INSTALL_LINE_NUMBER,
            body: installer_body()?,
        });
        self.lines.sort_by_key(|line| line.number);

        let mut raw = self.compact_unpadded_rebuild()?;
        let mut comment_bytes_removed = 0usize;
        if let Some(max_program_bytes) = max_program_bytes {
            if raw.len() > max_program_bytes {
                let needed = raw.len() - max_program_bytes;
                comment_bytes_removed = self.trim_rem_comment_bytes(needed);
                if comment_bytes_removed != needed {
                    bail!(
                        "resident bootstrap exceeds its existing FAT chain by {needed} byte(s), but only {comment_bytes_removed} removable REM comment byte(s) were available"
                    );
                }
                raw = self.compact_unpadded_rebuild()?;
            }
        }
        let program_bytes = raw.len();
        let allocated_bytes = program_bytes.div_ceil(256) * 256;
        let mut bytes = raw;
        bytes.resize(allocated_bytes, 0);
        Ok(BootstrapResult {
            program_bytes,
            bytes,
            payload_line_start: PAYLOAD_LINE_START,
            allocated_bytes,
            comment_bytes_removed,
        })
    }

    fn relocate_leading_clear(&mut self) -> Result<()> {
        let matches = self
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.body.first() == Some(&TOKEN_CLEAR))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            bail!(
                "expected exactly one leading CLEAR statement for bootstrap relocation, found {}",
                matches.len()
            );
        }
        let line_index = matches[0];
        let colon = self.lines[line_index]
            .body
            .iter()
            .position(|byte| *byte == b':')
            .context("leading CLEAR statement is not followed by the startup body")?;
        let mut clear_body = self.lines[line_index].body[..colon].to_vec();
        clear_body.push(0);
        let startup_body = self.lines[line_index].body[colon + 1..].to_vec();
        if startup_body.is_empty() || startup_body.last() != Some(&0) {
            bail!("startup body after the leading CLEAR statement is malformed");
        }
        self.lines[line_index].body = startup_body;
        self.lines.push(BasicLine {
            number: PRELOAD_CLEAR_LINE_NUMBER,
            body: clear_body,
        });
        Ok(())
    }

    pub fn install_comment_probe(&mut self) -> Result<Vec<u8>> {
        if self
            .lines
            .iter()
            .any(|line| line.number == LOADER_LINE_START)
        {
            bail!("BASIC comment probe line number is already occupied");
        }
        self.lines.push(BasicLine {
            number: LOADER_LINE_START,
            body: [vec![TOKEN_REM], b"RHPROBE".to_vec(), vec![0]].concat(),
        });
        self.lines.sort_by_key(|line| line.number);
        let raw = self.serialize_with_trailing(&self.trailing)?;
        let allocated_bytes = raw.len().div_ceil(256) * 256;
        let mut bytes = raw;
        bytes.resize(allocated_bytes, 0);
        Ok(bytes)
    }

    pub fn install_payload_probe(&mut self, renderer: &[u8]) -> Result<Vec<u8>> {
        if self.lines.iter().any(|line| {
            (PROBE_PAYLOAD_LINE_START..PROBE_PAYLOAD_LINE_START + 160).contains(&line.number)
        }) {
            bail!("BASIC payload probe line number is already occupied");
        }
        for (index, segment) in renderer.chunks(PAYLOAD_SEGMENT_BYTES).enumerate() {
            let mut body = vec![TOKEN_REM];
            body.extend_from_slice(PROBE_PAYLOAD_MARKER);
            for value in segment {
                body.push((value >> 4) + PAYLOAD_NIBBLE_BASE);
                body.push((value & 0x0F) + PAYLOAD_NIBBLE_BASE);
            }
            body.push(0);
            self.lines.push(BasicLine {
                number: PROBE_PAYLOAD_LINE_START + u16::try_from(index)? * PAYLOAD_LINE_STEP,
                body,
            });
        }
        self.lines.sort_by_key(|line| line.number);
        let raw = self.serialize_with_trailing(&self.trailing)?;
        let allocated_bytes = raw.len().div_ceil(256) * 256;
        let mut bytes = raw;
        bytes.resize(allocated_bytes, 0);
        Ok(bytes)
    }

    fn serialize_with_trailing(&self, trailing: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        for line in &self.lines {
            if line.body.len() > 255 {
                bail!("BASIC line {} body exceeds 255 bytes", line.number);
            }
            let next_position = output
                .len()
                .checked_add(4 + line.body.len())
                .context("BASIC line size overflow")?;
            let next_pointer = u16::try_from(next_position + 1)
                .context("BASIC program exceeds its 16-bit line pointer space")?;
            output.extend_from_slice(&next_pointer.to_le_bytes());
            output.extend_from_slice(&line.number.to_le_bytes());
            output.extend_from_slice(&line.body);
        }
        output.extend_from_slice(&[0, 0]);
        output.extend_from_slice(trailing);
        Ok(output)
    }
}

/// Number of bytes carried by an N88 BASIC token after the token byte itself.
/// Payload bytes are opaque and may equal statement tokens such as REM (8Fh).
fn n88_token_payload_len(token: u8) -> usize {
    match token {
        // Octal, hexadecimal, stored line reference, and line-number constants.
        0x0B..=0x0E => 2,
        // Unsigned one-byte integer constant.
        INTEGER_U8 => 1,
        // Signed integer, single precision, and double precision constants.
        0x1C => 2,
        0x1D => 4,
        0x1F => 8,
        // N88 functions and extended operators use FFh plus one token byte.
        EXTENDED_TOKEN_PREFIX => 1,
        _ => 0,
    }
}

fn basic_content_end(body: &[u8]) -> Option<usize> {
    body.len()
        .checked_sub(1)
        .filter(|_| body.last() == Some(&0))
}

fn ensure_line_size(number: u16, size: usize) -> Result<()> {
    if size > 255 {
        bail!("BASIC line {number} grows to {size} bytes, exceeding the 255-byte line limit");
    }
    Ok(())
}

fn decode_loader_body(segment_size: usize, destination: u16) -> Result<Vec<u8>> {
    let final_index = segment_size
        .checked_sub(1)
        .context("renderer segment must not be empty")?;
    let final_index = u16::try_from(final_index).context("renderer segment index exceeds u16")?;
    let mut body = Vec::new();

    // FOR RH%=0 TO segment_size-1
    body.extend_from_slice(&[TOKEN_FOR, b'R', b'H', b'%', TOKEN_EQUALS, COMPACT_ZERO]);
    body.push(TOKEN_TO);
    push_u16_literal(&mut body, final_index);
    body.push(b':');
    // POKE destination+RH%,(ASC(MID$(RZ$,RH%*2+1,1))-65)*16
    //                      +ASC(MID$(RZ$,RH%*2+2,1))-65
    body.push(TOKEN_POKE);
    push_u16_literal(&mut body, destination);
    body.extend_from_slice(&[TOKEN_PLUS, b'R', b'H', b'%', b',', b'(']);
    push_string_nibble(&mut body, COMPACT_ONE);
    body.extend_from_slice(&[b')', TOKEN_MULTIPLY]);
    push_u8_literal(&mut body, 16);
    body.push(TOKEN_PLUS);
    push_string_nibble(&mut body, COMPACT_TWO);
    body.push(b':');
    body.extend_from_slice(&[TOKEN_NEXT, b'R', b'H', b'%', 0]);
    Ok(body)
}

fn push_string_nibble(body: &mut Vec<u8>, one_based_position: u8) {
    body.extend_from_slice(&TOKEN_ASC_PREFIX);
    body.push(b'(');
    body.extend_from_slice(&TOKEN_MID_PREFIX);
    body.push(b'(');
    body.extend_from_slice(PAYLOAD_VARIABLE);
    body.extend_from_slice(&[b',', b'R', b'H', b'%', TOKEN_MULTIPLY, COMPACT_TWO]);
    body.extend_from_slice(&[
        TOKEN_PLUS,
        one_based_position,
        b',',
        COMPACT_ONE,
        b')',
        b')',
    ]);
    body.push(TOKEN_MINUS);
    push_u8_literal(body, PAYLOAD_NIBBLE_BASE);
}

fn installer_body() -> Result<Vec<u8>> {
    let state_count = STATE_END
        .checked_sub(STATE_START)
        .context("invalid renderer state range")?;
    let mut body = Vec::new();
    // Clear BDF0h..BDFBh before publishing the output vector.
    body.extend_from_slice(&[TOKEN_FOR, b'R', b'H', b'%', TOKEN_EQUALS, COMPACT_ZERO]);
    body.extend_from_slice(&[b' ', TOKEN_TO, b' ']);
    push_u8_literal(
        &mut body,
        u8::try_from(state_count).context("state count exceeds u8")?,
    );
    body.push(b':');
    body.extend_from_slice(&[TOKEN_POKE, b' ']);
    push_u16_literal(&mut body, STATE_START);
    body.extend_from_slice(&[
        TOKEN_PLUS,
        b'R',
        b'H',
        b'%',
        b',',
        COMPACT_ZERO,
        b':',
        TOKEN_NEXT,
        b' ',
        b'R',
        b'H',
        b'%',
        b':',
    ]);

    // ED42: JP BC00h.  Write opcode last so a partial vector is never active.
    push_poke(&mut body, 0xED43, 0x00);
    body.push(b':');
    push_poke(&mut body, 0xED44, 0xBC);
    body.push(b':');
    push_poke(&mut body, 0xED42, 0xC3);
    body.push(0);
    Ok(body)
}

fn push_poke(output: &mut Vec<u8>, address: u16, value: u8) {
    output.extend_from_slice(&[TOKEN_POKE, b' ']);
    push_u16_literal(output, address);
    output.push(b',');
    push_u8_literal(output, value);
}

fn push_u16_literal(output: &mut Vec<u8>, value: u16) {
    output.push(INTEGER_U16);
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_u8_literal(output: &mut Vec<u8>, value: u8) {
    if value == 0 {
        output.push(COMPACT_ZERO);
    } else {
        output.extend_from_slice(&[INTEGER_U8, value]);
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let raw = bytes
        .get(offset..offset + 2)
        .with_context(|| format!("BASIC u16 read at 0x{offset:X} is out of bounds"))?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_program() -> Vec<u8> {
        let lines = vec![
            BasicLine {
                number: 200,
                body: vec![
                    TOKEN_CLEAR,
                    b',',
                    INTEGER_U16,
                    0xFF,
                    0xCE,
                    b':',
                    TOKEN_REM,
                    b'Y',
                    0,
                ],
            },
            BasicLine {
                number: 210,
                body: [vec![TOKEN_REM], vec![b'X'; 32], vec![0]].concat(),
            },
        ];
        BasicProgram {
            lines,
            trailing: vec![0xAA, 0x55],
        }
        .byte_exact_rebuild()
        .expect("sample should serialize")
    }

    #[test]
    fn basic_roundtrip_is_byte_exact() {
        let source = sample_program();
        let parsed = BasicProgram::parse(&source).expect("sample should parse");
        assert_eq!(parsed.byte_exact_rebuild().unwrap(), source);
    }

    #[test]
    fn bootstrap_preserves_opaque_tail_and_embeds_payload() {
        let source = sample_program();
        let mut parsed = BasicProgram::parse(&source).unwrap();
        parsed.patch_memory_ceiling().unwrap();
        let result = parsed.install_bootstrap(&[0xF5, 0x00, 0xC9], None).unwrap();
        assert_eq!(&result.bytes[result.bytes.len() - 2..], &[0, 0]);
        let rebuilt = BasicProgram::parse(&result.bytes).unwrap();
        assert_eq!(
            rebuilt.embedded_payload().unwrap().unwrap(),
            [0xF5, 0, 0xC9]
        );
        assert!(result.bytes.windows(2).any(|window| window == [0xAA, 0x55]));
        assert_eq!(
            rebuilt
                .lines
                .iter()
                .find(|line| line.number == 191)
                .unwrap()
                .body[0],
            TOKEN_CLEAR
        );
        assert_ne!(
            rebuilt
                .lines
                .iter()
                .find(|line| line.number == 200)
                .unwrap()
                .body[0],
            TOKEN_CLEAR
        );
    }

    #[test]
    fn bootstrap_supports_six_segments_and_trims_only_rem_capacity() {
        let payload = (0..572).map(|index| index as u8).collect::<Vec<_>>();
        let source = sample_program();

        let mut unlimited = BasicProgram::parse(&source).unwrap();
        unlimited.patch_memory_ceiling().unwrap();
        let unlimited_result = unlimited.install_bootstrap(&payload, None).unwrap();
        assert_eq!(
            BasicProgram::parse(&unlimited_result.bytes)
                .unwrap()
                .embedded_payload()
                .unwrap()
                .unwrap(),
            payload
        );

        let mut limited = BasicProgram::parse(&source).unwrap();
        limited.patch_memory_ceiling().unwrap();
        let result = limited
            .install_bootstrap(&payload, Some(unlimited_result.program_bytes - 5))
            .unwrap();
        assert_eq!(result.comment_bytes_removed, 5);
        assert_eq!(result.program_bytes, unlimited_result.program_bytes - 5);
    }

    #[test]
    fn quoted_string_scan_preserves_structural_offsets() {
        let program = BasicProgram {
            lines: vec![BasicLine {
                number: 100,
                body: vec![0xFF, 0xDF, b'"', b'A', b'"', b':', b'X', b'"', b'B', 0],
            }],
            trailing: Vec::new(),
        };
        let strings = program.quoted_strings().unwrap();
        assert_eq!(strings.len(), 2);
        assert_eq!(strings[0].bytes, b"A");
        assert_eq!(strings[0].file_offset, 7);
        assert_eq!(strings[0].statement_offset, 0);
        assert!(strings[0].terminated);
        assert_eq!(strings[1].bytes, b"B");
        assert_eq!(strings[1].file_offset, 12);
        assert_eq!(strings[1].statement_offset, 6);
        assert!(!strings[1].terminated);
    }

    #[test]
    fn compact_rebuild_drops_only_zero_allocation_padding() {
        let program = BasicProgram {
            lines: vec![BasicLine {
                number: 100,
                body: vec![0],
            }],
            trailing: [vec![0xAA, 0x55], vec![0; 600]].concat(),
        };
        let compact = program.compact_sector_rebuild().unwrap();
        assert_eq!(compact.len(), 256);
        let reparsed = BasicProgram::parse(&compact).unwrap();
        assert!(reparsed
            .byte_exact_rebuild()
            .unwrap()
            .windows(2)
            .any(|pair| pair == [0xAA, 0x55]));
    }

    #[test]
    fn rem_comment_trimming_preserves_token_line_and_quoted_bytes() {
        let mut program = BasicProgram {
            lines: vec![BasicLine {
                number: 100,
                body: vec![
                    b'Q', b'$', 0xF1, b'"', 0x8F, b'"', b':', TOKEN_REM, b'A', b'B', b'C', 0,
                ],
            }],
            trailing: Vec::new(),
        };
        assert_eq!(program.trim_rem_comment_bytes(2), 2);
        assert_eq!(
            program.lines[0].body,
            [b'Q', b'$', 0xF1, b'"', 0x8F, b'"', b':', TOKEN_REM, b'A', 0]
        );
        assert_eq!(program.lines[0].number, 100);
    }

    #[test]
    fn rem_comment_trimming_skips_numeric_and_extended_token_payloads() {
        let executable = vec![
            0xFE,
            b'(',
            INTEGER_U8,
            203,
            b',',
            INTEGER_U8,
            TOKEN_REM,
            b')',
            b',',
            b'"',
            b'(',
            b'7',
            b')',
            b'X',
            b'"',
            b',',
            0x14,
            b',',
            0x13,
            b',',
            0x12,
            b':',
            EXTENDED_TOKEN_PREFIX,
            TOKEN_REM,
        ];
        let mut body = executable.clone();
        body.extend_from_slice(&[b':', TOKEN_REM, b'A', b'B', b'C', 0]);
        let mut program = BasicProgram {
            lines: vec![BasicLine { number: 420, body }],
            trailing: Vec::new(),
        };

        assert_eq!(program.trim_rem_comment_bytes(2), 2);
        let mut expected = executable;
        expected.extend_from_slice(&[b':', TOKEN_REM, b'A', 0]);
        assert_eq!(program.lines[0].body, expected);
    }

    #[test]
    fn quoted_replacement_updates_only_payload() {
        let mut program = BasicProgram {
            lines: vec![BasicLine {
                number: 100,
                body: vec![0x91, b'"', b'A', b'B', b'"', 0],
            }],
            trailing: Vec::new(),
        };
        program
            .replace_quoted_string(0, 2, b"AB", b"XYZ", true)
            .unwrap();
        assert_eq!(
            program.lines[0].body,
            [0x91, b'"', b'X', b'Y', b'Z', b'"', 0]
        );
    }

    #[test]
    fn cls_instrumentation_patches_only_executable_tokens() {
        let mut program = BasicProgram {
            lines: vec![
                BasicLine {
                    number: 100,
                    body: vec![
                        TOKEN_CLS,
                        b':',
                        TOKEN_PRINT,
                        b'"',
                        TOKEN_CLS,
                        b'"',
                        b':',
                        TOKEN_CLS,
                        0,
                    ],
                },
                BasicLine {
                    number: 110,
                    body: vec![
                        INTEGER_U8,
                        TOKEN_CLS,
                        b':',
                        EXTENDED_TOKEN_PREFIX,
                        TOKEN_CLS,
                        b':',
                        TOKEN_REM,
                        TOKEN_CLS,
                        0,
                    ],
                },
                BasicLine {
                    number: 120,
                    body: vec![TOKEN_POLL[0], TOKEN_POLL[1], b' ', TOKEN_CLS, 0],
                },
                BasicLine {
                    number: 130,
                    body: vec![
                        TOKEN_DATA, b'A', b',', TOKEN_CLS, b',', b'"', TOKEN_CLS, b':', TOKEN_CLS,
                        b'"', b':', TOKEN_CLS, 0,
                    ],
                },
            ],
            trailing: Vec::new(),
        };

        assert_eq!(program.instrument_cls().unwrap(), 3);
        let expected = [
            CLS_INSTRUMENTATION.as_slice(),
            &[TOKEN_CLS, b':', TOKEN_PRINT, b'"', TOKEN_CLS, b'"', b':'],
            CLS_INSTRUMENTATION.as_slice(),
            &[TOKEN_CLS, 0],
        ]
        .concat();
        assert_eq!(program.lines[0].body, expected);
        assert_eq!(
            program.lines[1].body,
            [
                INTEGER_U8,
                TOKEN_CLS,
                b':',
                EXTENDED_TOKEN_PREFIX,
                TOKEN_CLS,
                b':',
                TOKEN_REM,
                TOKEN_CLS,
                0,
            ]
        );
        assert_eq!(
            program.lines[2].body,
            [TOKEN_POLL[0], TOKEN_POLL[1], b' ', TOKEN_CLS, 0]
        );
        assert_eq!(
            program.lines[3].body,
            [
                &[
                    TOKEN_DATA, b'A', b',', TOKEN_CLS, b',', b'"', TOKEN_CLS, b':', TOKEN_CLS,
                    b'"', b':'
                ][..],
                CLS_INSTRUMENTATION.as_slice(),
                &[TOKEN_CLS, 0],
            ]
            .concat()
        );
    }

    #[test]
    fn cls_instrumentation_rejects_oversized_lines() {
        let mut body = vec![b'X'; 254];
        body[0] = TOKEN_CLS;
        body.push(0);
        let mut program = BasicProgram {
            lines: vec![BasicLine { number: 100, body }],
            trailing: Vec::new(),
        };
        assert!(program
            .instrument_cls()
            .unwrap_err()
            .to_string()
            .contains("255-byte line limit"));
    }

    #[test]
    fn profiled_page_clear_requires_the_expected_statement_prefix() {
        let expected_prefix = [0xCB, b' ', 0x17, b':', b'Q', b'$', TOKEN_EQUALS];
        let original = [expected_prefix.as_slice(), &[b'"', b'A', b'"', 0]].concat();
        let mut program = BasicProgram {
            lines: vec![BasicLine {
                number: 140,
                body: original.clone(),
            }],
            trailing: Vec::new(),
        };

        program
            .instrument_page_clear_before_line(140, &expected_prefix)
            .unwrap();
        assert_eq!(
            program.lines[0].body,
            [CLS_INSTRUMENTATION.as_slice(), original.as_slice()].concat()
        );

        let mut changed = BasicProgram {
            lines: vec![BasicLine {
                number: 140,
                body: vec![TOKEN_PRINT, 0],
            }],
            trailing: Vec::new(),
        };
        assert!(changed
            .instrument_page_clear_before_line(140, &expected_prefix)
            .unwrap_err()
            .to_string()
            .contains("profiled statement prefix"));
    }

    #[test]
    fn native_ascii_echo_instruments_dynamic_print_a_dollar() {
        let mut program = BasicProgram {
            lines: vec![
                BasicLine {
                    number: 100,
                    body: vec![
                        TOKEN_PRINT,
                        b' ',
                        b'A',
                        b'$',
                        b':',
                        TOKEN_POLL[0],
                        TOKEN_POLL[1],
                        b' ',
                        TOKEN_CLS,
                        0,
                    ],
                },
                BasicLine {
                    number: 110,
                    body: vec![TOKEN_PRINT, b' ', b'B', b'$', b':', TOKEN_CLS, 0],
                },
                BasicLine {
                    number: 120,
                    body: vec![
                        0x8B,
                        b'X',
                        TOKEN_EQUALS,
                        COMPACT_ONE,
                        b' ',
                        TOKEN_PRINT,
                        b' ',
                        b'A',
                        b'$',
                        0,
                    ],
                },
            ],
            trailing: Vec::new(),
        };

        assert_eq!(program.instrument_native_ascii_echo().unwrap(), 2);
        assert_eq!(
            program.lines[0].body,
            [
                &[TOKEN_PRINT][..],
                NATIVE_ASCII_PREFIX.as_slice(),
                &[
                    b' ',
                    b'A',
                    b'$',
                    b':',
                    TOKEN_POLL[0],
                    TOKEN_POLL[1],
                    b' ',
                    TOKEN_CLS,
                    0,
                ],
            ]
            .concat()
        );
        assert!(program.lines[2]
            .body
            .windows(NATIVE_ASCII_PREFIX.len() + 1)
            .any(|window| { window[0] == TOKEN_PRINT && window[1..] == NATIVE_ASCII_PREFIX }));
        assert_eq!(program.instrument_cls().unwrap(), 1);
    }

    #[test]
    fn embedded_zero_in_numeric_payload_does_not_end_the_line() {
        let mut program = BasicProgram {
            lines: vec![BasicLine {
                number: 220,
                body: vec![
                    INTEGER_U16,
                    0xE0,
                    0x00,
                    b':',
                    TOKEN_PRINT,
                    b'"',
                    b'A',
                    b'"',
                    b':',
                    TOKEN_CLS,
                    0,
                ],
            }],
            trailing: Vec::new(),
        };

        assert_eq!(program.quoted_strings().unwrap()[0].bytes, b"A");
        assert_eq!(program.instrument_cls().unwrap(), 1);
        assert!(program.lines[0]
            .body
            .windows(CLS_INSTRUMENTATION.len() + 1)
            .any(|window| {
                window[..CLS_INSTRUMENTATION.len()] == CLS_INSTRUMENTATION
                    && window[CLS_INSTRUMENTATION.len()] == TOKEN_CLS
            }));
    }

    #[test]
    fn ignored_space_trimming_protects_text_data_and_comments() {
        let mut program = BasicProgram {
            lines: vec![BasicLine {
                number: 100,
                body: vec![
                    b' ',
                    TOKEN_PRINT,
                    b' ',
                    b'"',
                    b'A',
                    b' ',
                    b'B',
                    b'"',
                    b':',
                    TOKEN_DATA,
                    b' ',
                    b'A',
                    b' ',
                    b'B',
                    b':',
                    b' ',
                    TOKEN_REM,
                    b' ',
                    b'C',
                    0,
                ],
            }],
            trailing: Vec::new(),
        };

        assert_eq!(program.trim_ignored_space_bytes(20), 3);
        assert_eq!(
            program.lines[0].body,
            [
                TOKEN_PRINT,
                b'"',
                b'A',
                b' ',
                b'B',
                b'"',
                b':',
                TOKEN_DATA,
                b' ',
                b'A',
                b' ',
                b'B',
                b':',
                TOKEN_REM,
                b' ',
                b'C',
                0,
            ]
        );
    }
}
