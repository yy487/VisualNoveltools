use anyhow::{bail, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const LOAD_ADDRESS: u16 = 0xBC00;
pub const STATE_START: u16 = 0xBDF0;
pub const STATE_END: u16 = 0xBDFB;
const SECONDARY_CODE_ADDRESS: u16 = 0xBE00;

const PENDING_ADDRESS: u16 = 0xBDF0;
const BASE_LOW_ADDRESS: u16 = 0xBDF1;
const BASE_HIGH_ADDRESS: u16 = 0xBDF2;
const OVERFLOW_ADDRESS: u16 = 0xBDF3;
const LAST_GLYPH_ROW_ADDRESS: u16 = 0xBDF4;
const NATIVE_ASCII_PENDING_ADDRESS: u16 = 0xBDF5;
const PAGE_MARKER_ADDRESS: u16 = 0xBDF7;
const PAGE_START_ADDRESS: u16 = 0xBDF8;
const PAGE_END_ADDRESS: u16 = 0xBDFA;

const PAGE_CLEAR_PENDING: u8 = 4;
const PAGE_LINE_PENDING: u8 = 5;

pub const FIRST_SCREEN_ROW: u8 = 1;
pub const LAST_SAFE_GLYPH_ROW: u8 = 24;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LayoutState {
    last_glyph_row: Option<u8>,
    line_pending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphPlan {
    pub row: u8,
}

impl LayoutState {
    pub fn observe_line_feed(&mut self) {
        if self.last_glyph_row.is_some() {
            self.line_pending = true;
        }
    }

    pub fn observe_native_wrap(&mut self) {
        self.observe_line_feed();
    }

    pub fn observe_page_clear(&mut self) {
        self.line_pending = false;
        self.last_glyph_row = None;
    }

    pub fn prepare_glyph(&mut self, native_row: u8) -> Option<GlyphPlan> {
        let mut row = native_row;
        if self.line_pending {
            self.line_pending = false;
            if self
                .last_glyph_row
                .is_some_and(|previous| previous.checked_add(1) == Some(native_row))
            {
                row = row.checked_add(1)?;
            }
        }
        if !(FIRST_SCREEN_ROW..=LAST_SAFE_GLYPH_ROW).contains(&row) {
            return None;
        }
        self.last_glyph_row = Some(row);
        Some(GlyphPlan { row })
    }

    pub fn advance_custom_cursor(row: u8, column: u8, width: u8) -> Option<(u8, u8)> {
        if column < width {
            return Some((row, column + 1));
        }
        let next_row = row.checked_add(2)?;
        (next_row <= LAST_SAFE_GLYPH_ROW).then_some((next_row, 1))
    }
}

pub const PAGE_CLEAR_SENTINEL: [u8; 2] = [0xF8, 0x7E];
pub const NATIVE_ASCII_SENTINEL: [u8; 2] = [0xF8, 0x7D];

pub const EXPECTED_SIZE: usize = 583;
pub const EXPECTED_SHA256: &str =
    "67A683ACF182C889290DDF1CC8916B3768169DE0ABAB5962FF7693ED67BD456B";

const SAFE_LEADS: [u8; 16] = [
    0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE6, 0xE7, 0xE8, 0xEB, 0xEC, 0xED, 0xEE, 0xEF, 0xF6, 0xF7, 0xF8,
];

#[derive(Debug, Clone, Copy)]
enum FixupKind {
    Absolute,
    Relative,
}

#[derive(Debug)]
struct Fixup {
    operand_offset: usize,
    target: String,
    kind: FixupKind,
}

#[derive(Debug, Default)]
struct Z80Emitter {
    bytes: Vec<u8>,
    labels: HashMap<String, usize>,
    fixups: Vec<Fixup>,
}

impl Z80Emitter {
    fn emit(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn label(&mut self, name: impl Into<String>) -> Result<()> {
        let name = name.into();
        if self.labels.insert(name.clone(), self.bytes.len()).is_some() {
            bail!("duplicate renderer label {name}");
        }
        Ok(())
    }

    fn absolute(&mut self, opcode: u8, target: impl Into<String>) {
        self.emit(&[opcode, 0, 0]);
        self.fixups.push(Fixup {
            operand_offset: self.bytes.len() - 2,
            target: target.into(),
            kind: FixupKind::Absolute,
        });
    }

    fn relative(&mut self, opcode: u8, target: impl Into<String>) {
        self.emit(&[opcode, 0]);
        self.fixups.push(Fixup {
            operand_offset: self.bytes.len() - 1,
            target: target.into(),
            kind: FixupKind::Relative,
        });
    }

    fn pad_to(&mut self, address: u16) -> Result<()> {
        let target_offset = usize::from(address)
            .checked_sub(usize::from(LOAD_ADDRESS))
            .ok_or_else(|| anyhow::anyhow!("renderer padding target precedes its load address"))?;
        if self.bytes.len() > target_offset {
            bail!(
                "renderer code reaches 0x{:04X}, crossing padding target 0x{address:04X}",
                usize::from(LOAD_ADDRESS) + self.bytes.len()
            );
        }
        self.bytes.resize(target_offset, 0);
        Ok(())
    }

    fn finish(mut self) -> Result<Vec<u8>> {
        for fixup in self.fixups {
            let target_offset = *self
                .labels
                .get(&fixup.target)
                .ok_or_else(|| anyhow::anyhow!("unknown renderer label {}", fixup.target))?;
            match fixup.kind {
                FixupKind::Absolute => {
                    let target = usize::from(LOAD_ADDRESS) + target_offset;
                    let target = u16::try_from(target)?;
                    self.bytes[fixup.operand_offset..fixup.operand_offset + 2]
                        .copy_from_slice(&target.to_le_bytes());
                }
                FixupKind::Relative => {
                    let next_offset = fixup.operand_offset + 1;
                    let displacement =
                        isize::try_from(target_offset)? - isize::try_from(next_offset)?;
                    let displacement = i8::try_from(displacement).map_err(|_| {
                        anyhow::anyhow!(
                            "renderer relative branch to {} is out of range ({displacement})",
                            fixup.target
                        )
                    })?;
                    self.bytes[fixup.operand_offset] = displacement as u8;
                }
            }
        }
        Ok(self.bytes)
    }
}

pub fn resident_renderer() -> Result<Vec<u8>> {
    let bytes = build_resident_renderer()?;
    if bytes.len() != EXPECTED_SIZE {
        bail!(
            "embedded renderer has {} bytes, expected {EXPECTED_SIZE}",
            bytes.len()
        );
    }
    let hash = format!("{:X}", Sha256::digest(&bytes));
    if hash != EXPECTED_SHA256 {
        bail!("embedded renderer hash mismatch: expected {EXPECTED_SHA256}, actual {hash}");
    }
    Ok(bytes)
}

fn build_resident_renderer() -> Result<Vec<u8>> {
    let mut out = Z80Emitter::default();

    // ED42h entry: preserve the original AF/BC and classify only the 16
    // collision-free lead bytes. A pending byte stores table index + 1.
    out.label("entry")?;
    out.emit(&[0xF5, 0xC5, 0x47]); // PUSH AF / PUSH BC / LD B,A
    emit_ld_a_mem(&mut out, PENDING_ADDRESS);
    out.emit(&[0xB7]); // OR A
    out.absolute(0xC2, "have_lead"); // JP NZ
    out.emit(&[0x78, 0xFE, 0x0D]); // LD A,B / CP CR
    out.absolute(0xCA, "reset_overflow");
    out.emit(&[0x78, 0xFE, 0x0A]); // LD A,B / CP LF
    out.absolute(0xCA, "line_feed");

    // Printable ASCII is stored as one byte but rendered through reserved
    // page-zero KANJI1 slots.  Space remains the native half-width spacer.
    out.emit(&[0x78, 0xFE, 0x21]); // LD A,B; CP '!'
    out.relative(0x38, "classify_lead");
    out.emit(&[0xFE, 0x7F]); // CP DEL
    out.relative(0x30, "classify_lead");

    emit_ld_a_mem(&mut out, NATIVE_ASCII_PENDING_ADDRESS);
    out.emit(&[0xB7]);
    out.relative(0x28, "custom_ascii");
    out.emit(&[0xAF]);
    emit_ld_mem_a(&mut out, NATIVE_ASCII_PENDING_ADDRESS);
    out.absolute(0xC3, "chain_original");
    out.label("custom_ascii")?;
    out.emit(&[0x78]); // restore the printable byte in A

    out.emit(&[0xFE, 0x23]); // CP '#'
    out.relative(0x30, "ascii_slot_ready");
    out.emit(&[0x05]); // DEC B: ! -> trail 20h, quote -> trail 21h
    out.label("ascii_slot_ready")?;
    out.emit(&[0x0E, 0x01]); // C=lead table index 0 + 1
    out.absolute(0xC3, "normal_trail");

    // Classify the sparse safe-lead set through a table so the direct ASCII
    // path still fits below the resident state block at BDF0h.
    out.label("classify_lead")?;
    out.emit(&[0xE5]); // PUSH HL
    out.absolute(0x21, "safe_lead_table"); // LD HL,table
    out.emit(&[0x0E, 0x01]); // LD C,1
    out.label("lead_loop")?;
    out.emit(&[0x7E, 0xB8]); // LD A,(HL); CP B
    out.relative(0x28, "lead_found");
    out.emit(&[0x23, 0x0C, 0x79, 0xFE, 0x11]); // INC HL/C; LD A,C; CP 17
    out.relative(0x20, "lead_loop");
    out.emit(&[0xE1]); // POP HL
    out.label("original")?;
    // Native printable output can wrap at the last column. Remember that
    // one-row native move so the next 16x16 glyph can add the missing row.
    out.emit(&[0xE5]); // PUSH HL
    out.emit(&[0x78, 0xFE, 0x20]); // LD A,B; CP space
    out.relative(0x38, "restore_original_hl");
    out.emit(&[0x2A, 0x86, 0xEF]); // HL=(EF86), H=column
    emit_ld_a_mem(&mut out, 0xEF89);
    out.emit(&[0xBC]); // CP H
    out.relative(0x20, "restore_original_hl");
    emit_ld_a_mem(&mut out, LAST_GLYPH_ROW_ADDRESS);
    out.emit(&[0xB7]); // OR A
    out.relative(0x28, "restore_original_hl");
    emit_ld_a_mem(&mut out, PAGE_MARKER_ADDRESS);
    out.emit(&[0xFE, PAGE_CLEAR_PENDING]);
    out.relative(0x30, "restore_original_hl");
    out.emit(&[0x3E, PAGE_LINE_PENDING]);
    emit_ld_mem_a(&mut out, PAGE_MARKER_ADDRESS);
    out.label("restore_original_hl")?;
    out.emit(&[0xE1]); // POP HL
    out.label("chain_original")?;
    out.emit(&[0xC1, 0xF1, 0xC3, 0xE3, 0x91]); // POP BC/AF; JP 91E3h

    out.label("safe_lead_table")?;
    out.emit(&SAFE_LEADS);

    out.label("lead_found")?;
    out.emit(&[0x79, 0xE1]); // LD A,C; POP HL

    out.label("store_lead")?;
    emit_ld_mem_a(&mut out, PENDING_ADDRESS);
    out.emit(&[0xC1, 0xF1]); // POP BC / POP AF
    out.absolute(0xC3, "swallow");

    out.label("reset_overflow")?;
    out.emit(&[0xAF]); // XOR A
    emit_ld_mem_a(&mut out, OVERFLOW_ADDRESS);
    out.absolute(0xC3, "chain_original");

    // LF is executed by the original ROM and moves one native 8-pixel row.
    // Arm a conditional extra-row adjustment only when a custom glyph exists.
    // A pending page clear survives the CR/LF emitted after a wait prompt.
    out.label("line_feed")?;
    out.emit(&[0xAF]); // XOR A
    emit_ld_mem_a(&mut out, OVERFLOW_ADDRESS);
    emit_ld_a_mem(&mut out, PAGE_MARKER_ADDRESS);
    out.emit(&[0xFE, PAGE_CLEAR_PENDING]);
    out.absolute(0xD2, "chain_original"); // JP NC
    emit_ld_a_mem(&mut out, LAST_GLYPH_ROW_ADDRESS);
    out.emit(&[0xB7]);
    out.absolute(0xCA, "chain_original"); // JP Z
    out.emit(&[0x3E, PAGE_LINE_PENDING]);
    emit_ld_mem_a(&mut out, PAGE_MARKER_ADDRESS);
    out.absolute(0xC3, "chain_original");

    out.label("have_lead")?;
    out.emit(&[0x4F, 0xAF]); // LD C,A / XOR A
    emit_ld_mem_a(&mut out, PENDING_ADDRESS);
    out.emit(&[0x78, 0xB7]); // LD A,B / OR A
    out.relative(0x28, "invalid_trail");

    out.emit(&[0x79, 0xFE, 0x10]); // LD A,C; CP sentinel lead index + 1
    out.relative(0x20, "normal_trail");
    out.emit(&[0x78, 0xFE, NATIVE_ASCII_SENTINEL[1]]); // LD A,B; CP native trail
    out.relative(0x28, "native_ascii_command");
    out.emit(&[0xFE, PAGE_CLEAR_SENTINEL[1]]); // CP clear trail
    out.relative(0x20, "normal_trail");
    // The command is emitted immediately before the game's CLS statement.
    // Clear the old translated layer now, before CLS can draw the next scene.
    out.emit(&[0xD5, 0xE5, 0xF3]); // PUSH DE/HL; DI
    out.absolute(0xCD, "clear_previous_page");
    out.emit(&[0xFB, 0xE1, 0xD1, 0xC1, 0xF1]); // EI; restore HL/DE/BC/AF
    out.absolute(0xC3, "swallow");

    out.label("native_ascii_command")?;
    out.emit(&[0x3E, 0x01]);
    emit_ld_mem_a(&mut out, NATIVE_ASCII_PENDING_ADDRESS);
    out.emit(&[0xC1, 0xF1]);
    out.absolute(0xC3, "swallow");

    // base word = ((((lead table index) << 8) | trail) << 4)
    out.label("normal_trail")?;
    out.emit(&[0x78, 0x07, 0x07, 0x07, 0x07, 0xE6, 0xF0]);
    emit_ld_mem_a(&mut out, BASE_LOW_ADDRESS);
    out.emit(&[0x78, 0x0F, 0x0F, 0x0F, 0x0F, 0xE6, 0x0F, 0x47]);
    out.emit(&[0x79, 0x3D, 0x07, 0x07, 0x07, 0x07, 0xB0]);
    emit_ld_mem_a(&mut out, BASE_HIGH_ADDRESS);

    out.emit(&[0xC1, 0xF1, 0xF5, 0xC5, 0xD5, 0xE5]); // restore, then preserve AF/BC/DE/HL
    emit_ld_a_mem(&mut out, OVERFLOW_ADDRESS);
    out.emit(&[0xB7]);
    out.relative(0x20, "custom_done");
    out.emit(&[0xF3]); // DI
    out.absolute(0xCD, "draw_glyph");
    out.emit(&[0xFB]); // EI; draw_glyph restored port 5Fh
    out.absolute(0xCD, "advance_cursor");

    out.label("custom_done")?;
    out.emit(&[0xE1, 0xD1, 0xC1, 0xF1]);
    out.absolute(0xC3, "swallow");

    out.label("invalid_trail")?;
    out.emit(&[0xC1, 0xF1]);

    out.label("swallow")?;
    out.absolute(0xC3, "return_dispatch");

    // Draw one 16x16 glyph at the game's current cursor. If the original ROM
    // moved exactly one native row after LF/wrap, add the second 8-pixel row.
    // Explicit LOCATE/CLS jumps do not match that relation and remain intact.
    out.label("draw_glyph")?;
    out.emit(&[0x2A, 0x86, 0xEF]); // HL=(EF86)
    emit_ld_a_mem(&mut out, PAGE_MARKER_ADDRESS);
    out.emit(&[0xFE, PAGE_LINE_PENDING]);
    out.relative(0x20, "cursor_row");
    out.emit(&[0xAF]);
    emit_ld_mem_a(&mut out, PAGE_MARKER_ADDRESS);
    emit_ld_a_mem(&mut out, LAST_GLYPH_ROW_ADDRESS);
    out.emit(&[0x3C, 0xBD]); // INC A; CP L
    out.relative(0x20, "cursor_row");
    out.emit(&[0x2C, 0xCD, 0x94, 0x44]); // INC L; store EF86h
    out.label("cursor_row")?;
    out.emit(&[0x7D, 0xFE, LAST_SAFE_GLYPH_ROW + 1]); // A=L; CP 25
    out.absolute(0xD2, "mark_overflow"); // JP NC
    emit_ld_mem_a(&mut out, LAST_GLYPH_ROW_ADDRESS);

    out.label("cursor_ready")?;
    // HL = C000h + (row-1)*640 + (column-1)*2; BC retains x offset.
    out.emit(&[
        0x7C, 0x3D, 0x87, 0x4F, 0x06, 0x00, 0x7D, 0x3D, 0x6F, 0x26, 0x00, 0x29, 0x29, 0x29, 0x29,
        0x29, 0x29, 0x29, 0x54, 0x5D, 0x29, 0x29, 0x19, 0x09, 0x11, 0x00, 0xC0, 0x19,
    ]);

    // Record the min/max full-width text-row band. Explicit LOCATE can move in
    // either direction, so retaining only the first/latest row is insufficient.
    out.emit(&[0xE5, 0xB7, 0xED, 0x42]); // PUSH HL; OR A; SBC HL,BC
    emit_ld_a_mem(&mut out, PAGE_END_ADDRESS + 1);
    out.emit(&[0xB7]);
    out.relative(0x20, "page_existing");
    emit_ld_mem_hl(&mut out, PAGE_START_ADDRESS);
    out.emit(&[0x11, 0x00, 0x05, 0x19]); // DE=1280; ADD HL,DE
    emit_ld_mem_hl(&mut out, PAGE_END_ADDRESS);
    out.relative(0x18, "page_recorded");

    out.label("page_existing")?;
    emit_ld_de_mem(&mut out, PAGE_START_ADDRESS);
    // Row bases are 0280h apart, so their high bytes are unique over rows 1-24.
    out.emit(&[0x7C, 0xBA]); // LD A,H; CP D
    out.relative(0x30, "page_start_ready"); // candidate >= recorded start
    emit_ld_mem_hl(&mut out, PAGE_START_ADDRESS);
    out.label("page_start_ready")?;
    out.emit(&[0x11, 0x00, 0x05, 0x19]); // candidate end
    emit_ld_de_mem(&mut out, PAGE_END_ADDRESS);
    out.emit(&[0x7C, 0xBA]); // LD A,H; CP D
    out.relative(0x38, "page_recorded"); // candidate end < recorded end
    emit_ld_mem_hl(&mut out, PAGE_END_ADDRESS);
    out.label("page_recorded")?;
    out.emit(&[0xE1]); // POP HL

    out.emit(&[0x06, 0x10]); // 16 scanlines
    out.label("draw_row")?;
    out.emit(&[0x3E, 0x10, 0x90, 0x57]);
    emit_ld_a_mem(&mut out, BASE_LOW_ADDRESS);
    out.emit(&[0x82, 0x0E, 0xE8, 0xED, 0x79]);
    emit_ld_a_mem(&mut out, BASE_HIGH_ADDRESS);
    out.emit(&[0xCE, 0x00, 0x0C, 0xED, 0x79]);
    out.emit(&[0xED, 0x50, 0x0D, 0xED, 0x58]);
    out.emit(&[0xAF, 0xD3, 0x5C, 0x72, 0x23, 0x73, 0x2B]);
    out.emit(&[0xD3, 0x5D, 0x72, 0x23, 0x73, 0x2B]);
    out.emit(&[0xD3, 0x5E, 0x72, 0x23, 0x73, 0x2B]);
    out.emit(&[0x7D, 0xC6, 0x50, 0x6F]);
    out.relative(0x30, "no_scanline_carry");
    out.emit(&[0x24]);
    out.label("no_scanline_carry")?;
    out.relative(0x10, "draw_row");
    out.relative(0x18, "draw_done");

    out.label("mark_overflow")?;
    out.emit(&[0x3E, 0x01]);
    emit_ld_mem_a(&mut out, OVERFLOW_ADDRESS);
    out.label("draw_done")?;
    out.emit(&[0xAF, 0xD3, 0x5F, 0xC9]);

    out.label("advance_cursor")?;
    out.emit(&[0x2A, 0x86, 0xEF, 0x3A, 0x89, 0xEF, 0xBC]);
    out.relative(0x20, "advance_column");
    out.emit(&[0x7D, 0xFE, 0x18]);
    out.absolute(0xD2, "advance_overflow"); // JP NC
    out.emit(&[0x26, 0x01, 0x2C, 0x2C]);
    out.relative(0x18, "store_cursor");
    out.label("advance_column")?;
    out.emit(&[0x24]);
    out.relative(0x18, "store_cursor");
    out.label("advance_overflow")?;
    out.emit(&[0x3E, 0x01]);
    emit_ld_mem_a(&mut out, OVERFLOW_ADDRESS);
    out.emit(&[0xC9]);
    out.label("store_cursor")?;
    out.emit(&[0xCD, 0x94, 0x44, 0xC9]);

    // Return custom output through either N88's RST 18h wrapper or an ordinary
    // direct CALL ED42h. Only the wrapper has return 001Ch plus a saved AF word.
    out.pad_to(SECONDARY_CODE_ADDRESS)?;
    out.label("return_dispatch")?;
    out.emit(&[
        0xE3, 0xF5, // EX (SP),HL exposes the return word; PUSH AF
        0x7C, 0xB7, // high byte == 00h?
    ]);
    out.relative(0x20, "direct_return");
    out.emit(&[0x7D, 0xFE, 0x1C]); // low byte == 1Ch?
    out.relative(0x20, "direct_return");
    out.emit(&[0xF1, 0xE3, 0xF1, 0xF1, 0xC9]); // restore RST-saved AF/HL and RET
    out.label("direct_return")?;
    out.emit(&[0xF1, 0xE3, 0xC9]); // restore AF/HL; ordinary RET

    // Erase only the GVRAM text band touched by translated glyphs.
    out.label("clear_previous_page")?;
    emit_ld_a_mem(&mut out, PAGE_END_ADDRESS + 1);
    out.emit(&[0xB7]);
    out.relative(0x28, "clear_state");
    emit_ld_de_mem(&mut out, PAGE_END_ADDRESS);
    out.emit(&[0x0E, 0x5C, 0x06, 0x03]); // C=blue GVRAM port; B=three planes
    out.label("clear_plane")?;
    out.emit(&[0xAF, 0xED, 0x79]); // XOR A; OUT (C),A
    emit_ld_hl_mem(&mut out, PAGE_START_ADDRESS);
    out.absolute(0xCD, "clear_range");
    out.emit(&[0x0C]); // next GVRAM plane
    out.relative(0x10, "clear_plane");
    out.emit(&[0xAF, 0xD3, 0x5F]);
    out.label("clear_state")?;
    out.emit(&[0xAF, 0x21, 0xF0, 0xBD, 0x06, 0x0C]); // A=0; HL=state; B=12
    out.label("clear_state_byte")?;
    out.emit(&[0x77, 0x23]); // LD (HL),A; INC HL
    out.relative(0x10, "clear_state_byte");
    out.emit(&[0xC9]);

    out.label("clear_range")?;
    out.emit(&[0x36, 0x00, 0x23, 0x7C, 0xBA]); // (HL)=0; INC HL; A=H; CP D
    out.relative(0x20, "clear_range");
    out.emit(&[0x7D, 0xBB]); // A=L; CP E
    out.relative(0x20, "clear_range");
    out.emit(&[0xC9]);

    out.finish()
}

fn emit_ld_a_mem(out: &mut Z80Emitter, address: u16) {
    let [low, high] = address.to_le_bytes();
    out.emit(&[0x3A, low, high]);
}

fn emit_ld_mem_a(out: &mut Z80Emitter, address: u16) {
    let [low, high] = address.to_le_bytes();
    out.emit(&[0x32, low, high]);
}

fn emit_ld_hl_mem(out: &mut Z80Emitter, address: u16) {
    let [low, high] = address.to_le_bytes();
    out.emit(&[0x2A, low, high]);
}

fn emit_ld_de_mem(out: &mut Z80Emitter, address: u16) {
    let [low, high] = address.to_le_bytes();
    out.emit(&[0xED, 0x5B, low, high]);
}

fn emit_ld_mem_hl(out: &mut Z80Emitter, address: u16) {
    let [low, high] = address.to_le_bytes();
    out.emit(&[0x22, low, high]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_renderer_matches_pinned_size_and_hash() {
        let bytes = build_resident_renderer().expect("renderer should assemble");
        assert_eq!(bytes.len(), EXPECTED_SIZE);
        assert_eq!(format!("{:X}", Sha256::digest(&bytes)), EXPECTED_SHA256);
        assert_eq!(bytes[0], 0xF5);
        assert_eq!(bytes.last(), Some(&0xC9));
    }

    #[test]
    fn generated_renderer_matches_checked_in_hex() {
        let compact = include_str!("../assets/renderer_bc00.hex")
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        assert_eq!(compact.len() % 2, 0);
        let asset = (0..compact.len())
            .step_by(2)
            .map(|offset| u8::from_str_radix(&compact[offset..offset + 2], 16).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(build_resident_renderer().unwrap(), asset);
    }

    #[test]
    fn page_state_stays_outside_generated_code() {
        let bytes = build_resident_renderer().expect("renderer should assemble");
        let state_start = usize::from(STATE_START - LOAD_ADDRESS);
        let state_end = usize::from(STATE_END - LOAD_ADDRESS) + 1;
        assert!(bytes[state_start..state_end].iter().all(|byte| *byte == 0));
        assert!(usize::from(SECONDARY_CODE_ADDRESS) < usize::from(LOAD_ADDRESS) + bytes.len());
        const { assert!(STATE_END < 0xC000) };
    }

    #[test]
    fn layout_starts_at_the_game_cursor_and_advances_top_down() {
        let mut state = LayoutState::default();
        assert_eq!(state.prepare_glyph(1), Some(GlyphPlan { row: 1 }));
        state.observe_line_feed();
        assert_eq!(state.prepare_glyph(2).unwrap().row, 3);
        state.observe_line_feed();
        assert_eq!(state.prepare_glyph(4).unwrap().row, 5);
    }

    #[test]
    fn layout_preserves_the_original_22_24_window_when_the_game_uses_it() {
        let mut state = LayoutState::default();
        assert_eq!(state.prepare_glyph(22).unwrap().row, 22);
        state.observe_line_feed();
        assert_eq!(state.prepare_glyph(23).unwrap().row, 24);
    }

    #[test]
    fn explicit_cursor_reposition_is_not_forced_to_the_previous_grid() {
        let mut state = LayoutState::default();
        assert_eq!(state.prepare_glyph(22).unwrap().row, 22);
        state.observe_line_feed();
        assert_eq!(state.prepare_glyph(7).unwrap().row, 7);
    }

    #[test]
    fn page_clear_resets_layout_before_the_next_page() {
        let mut state = LayoutState::default();
        assert_eq!(state.prepare_glyph(18).unwrap().row, 18);
        state.observe_page_clear();
        state.observe_line_feed();
        assert_eq!(state.prepare_glyph(3), Some(GlyphPlan { row: 3 }));
    }

    #[test]
    fn native_and_custom_wraps_use_two_native_rows() {
        let mut state = LayoutState::default();
        assert_eq!(state.prepare_glyph(1).unwrap().row, 1);
        state.observe_native_wrap();
        assert_eq!(state.prepare_glyph(2).unwrap().row, 3);
        assert_eq!(LayoutState::advance_custom_cursor(3, 40, 40), Some((5, 1)));
        assert_eq!(LayoutState::advance_custom_cursor(24, 40, 40), None);
    }

    #[test]
    fn row_25_is_rejected_before_drawing() {
        let mut state = LayoutState::default();
        assert_eq!(state.prepare_glyph(25), None);
    }
}
