# -*- coding: utf-8 -*-
"""Opcode definitions for Refrain Blue / RP MES semantic assembler.

This module is intentionally conservative.  Only byte patterns that have been
confirmed in the current MES samples are given semantic relocation behavior.
All remaining bytes are preserved by the disassembler as .byte data records, so
round-trip rebuilding is byte-exact while unknown VM areas are not silently
misinterpreted as instructions.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Callable, Optional

DEFAULT_ENCODING = "cp932"


@dataclass(frozen=True)
class OpcodeSpec:
    mnemonic: str
    bytecode: bytes
    operands: tuple[str, ...] = ()
    length: int | str = 1
    description: str = ""


# Confirmed semantic opcodes / composite patterns.
# length="stringz" means bytecode prefix + encoded string bytes + NUL.
# length="abs32" means prefix + 4-byte little-endian absolute target.
OPCODES: dict[str, OpcodeSpec] = {
    "TEXT": OpcodeSpec(
        mnemonic="TEXT",
        bytecode=b"\x01",
        operands=("stringz",),
        length="stringz",
        description="Displayed/script string marker 01 <cp932-cstr> 00.",
    ),
    "SYSTEM_TEXT": OpcodeSpec(
        mnemonic="SYSTEM_TEXT",
        bytecode=b"\x02",
        operands=("stringz",),
        length="stringz",
        description="Known VM table entry for system text.  Parsed only when explicitly emitted.",
    ),
    "RUBY_TEXT": OpcodeSpec(
        mnemonic="RUBY_TEXT",
        bytecode=b"\x0B\x15\xFF\x01",
        operands=("stringz",),
        length="stringz",
        description="Furigana/ruby helper block prefix 0B 15 FF 01 <reading> 00.",
    ),
    "JUMP": OpcodeSpec(
        mnemonic="JUMP",
        bytecode=b"\x0A",
        operands=("abs32",),
        length="abs32",
        description="Confirmed OPX0A absolute jump: 0A <target:u32le>.",
    ),
    "CHOICE_DISPATCH": OpcodeSpec(
        mnemonic="CHOICE_DISPATCH",
        bytecode=b"\x09\x01\xFF",
        operands=("abs32",),
        length="abs32",
        description="Choice/menu dispatch head: 09 01 FF <exit:u32le>.",
    ),
    "CHOICE_BRANCH": OpcodeSpec(
        mnemonic="CHOICE_BRANCH",
        bytecode=b"\x0E\x02",
        operands=("imm8", "literal_ff00", "abs32"),
        length=9,
        description="Choice branch boundary: 0E 02 <index> FF 00 <branch_end:u32le>.",
    ),
    "NEW_LINE": OpcodeSpec(
        mnemonic="NEW_LINE",
        bytecode=b"\x13",
        operands=(),
        length=1,
        description="Display/new-line related command 13. The following byte may be another opcode such as TEXT/choice dispatch.",
    ),
}

# Numeric opcode notes from current analysis/reference tables.  These entries
# are documentation only unless they have a parser above.
VM_OPCODE_NOTES: dict[int, str] = {
    0x00: "RETURN / block terminator in reference table; often appears as data or padding in samples.",
    0x01: "TEXT S",
    0x02: "SYSTEM_TEXT S in reference table; also appears as parameter byte inside ruby/control blocks.",
    0x03: "B_FLAG_SET / HCG in reference table",
    0x04: "W_FLAG_SET / BCG",
    0x05: "EXT_B_FLAG_SET / CCG",
    0x06: "PC_FLAG_SET / CBCG",
    0x07: "A_FLAG_SET / CBCG",
    0x08: "G_FLAG_SET / CFCG or CBCG depending VM version",
    0x09: "PW_FLAG_SET in reference table; 09 01 FF <addr> is confirmed choice dispatch in samples",
    0x0A: "PB_FLAG_SET in reference table; 0A <addr> is confirmed OPX absolute jump in samples",
    0x0B: "JUMP_IF / CI in reference table; also prefixes ruby/helper control sequences",
    0x0C: "JUMP / I in reference table; not used as relocation target in this conservative tool yet",
    0x0D: "SYS / CV",
    0x0E: "CH_POS / V in reference table; 0E 02 ... <addr> is confirmed choice branch boundary",
    0x0F: "CALL / V",
    0x10: "MENU_SET / VI",
    0x11: "INTERRUPT / V",
    0x12: "SPEC_SYS / V",
    0x13: "NEW_LINE",
    0x14: "INTERRUPT_IF / CI",
    0x15: "MENU / variant-dependent; also appears in ruby prefix 0B 15 FF 01",
    0x16: "FLAG_D_SET / BCG",
    0x17: "MESSAGE / I in newer table",
    0x1F: "LABEL / I in newer table",
}



# EXE-backed VM model extracted from rp.EXE.c.
# Main loop: thunk_FUN_00406110.
# Expression reader: thunk_FUN_00413350.
TOPLEVEL_OPCODES = {
    0x00: {"mnemonic": "RETURN", "format": "", "handler": "thunk_FUN_00406110:return"},
    0x01: {"mnemonic": "TEXT", "format": "cstring", "handler": "thunk_FUN_0040e5f0"},
    0x02: {"mnemonic": "SYSTEM_TEXT", "format": "cstring", "handler": "thunk_FUN_00406ca0"},
    0x03: {"mnemonic": "SET_BYTE_VAR", "format": "u16le expr_list_00", "handler": "thunk_FUN_00406e30"},
    0x04: {"mnemonic": "SET_WORD_TABLE", "format": "u8 expr_list_00", "handler": "thunk_FUN_00406ea0"},
    0x05: {"mnemonic": "SET_BYTE_TABLE", "format": "expr expr_list_00", "handler": "thunk_FUN_00406ef0"},
    0x06: {"mnemonic": "SET_BYTE_TABLE_BASE", "format": "expr u8 expr_list_00", "handler": "thunk_FUN_00406f90"},
    0x07: {"mnemonic": "SET_WORD_TABLE_BASE", "format": "expr u8 expr_list_00", "handler": "thunk_FUN_00407010"},
    0x08: {"mnemonic": "SET_DWORD_TABLE_BASE", "format": "expr u8 expr_list_00", "handler": "thunk_FUN_004070a0"},
    0x09: {"mnemonic": "JUMP_IF_EXPR_NOT_1", "format": "expr_413350 target:u32le", "handler": "sub_407120"},
    0x0A: {"mnemonic": "JUMP", "format": "target:u32le", "handler": "thunk_FUN_004071b0"},
    0x0B: {"mnemonic": "SYS", "format": "sys_id_expr_413350 arglist_4059E0", "handler": "sub_407230"},
    0x0C: {"mnemonic": "SET_CONTEXT_STRING", "format": "arglist_4059E0", "handler": "sub_407460"},
    0x0D: {"mnemonic": "MENU_ACCUM", "format": "var", "handler": "thunk_FUN_00407520"},
    0x0E: {"mnemonic": "REGISTER_BRANCH_AND_SKIP", "format": "arglist_4059E0 target:u32le", "handler": "sub_4077C0"},
    0x0F: {"mnemonic": "DISPLAY_WAIT", "format": "argblock", "handler": "thunk_FUN_00407930"},
    0x10: {"mnemonic": "DISPLAY_CONTROL", "format": "", "handler": "thunk_FUN_004079f0"},
    0x11: {"mnemonic": "LINE_ADVANCE", "format": "u8", "handler": "thunk_FUN_0040b710"},
    0x12: {"mnemonic": "CALL_SAVE_JUMP", "format": "expr target:u32le", "handler": "thunk_FUN_0040b750"},
    0x13: {"mnemonic": "WAIT_COMMIT", "format": "none", "handler": "sub_40B7F0"},
    0x14: {"mnemonic": "SET_DWORD_TABLE", "format": "u8 expr_list_00", "handler": "thunk_FUN_00406f40"},
    0x15: {"mnemonic": "SYS_INLINE_TEXT", "format": "runtime", "handler": "thunk_FUN_0040e7f0"},
}

EXPR_TOKENS = {
    "default": "default byte value pushes a small literal; no immediate payload",
    0x80: "u8 payload; variable/table lookup then push",
    0xA0: "u8 payload; expression-indexed word lookup",
    0xC0: "u8 payload; expression-indexed byte lookup",
    "0xE0..0xF0": "runtime helper token; no immediate payload in sub_413350",
    0xF1: "u16 little-endian immediate",
    0xF2: "u32 little-endian immediate",
    0xF3: "u16 little-endian address/index source",
    0xF4: "expression-indexed byte source; no immediate payload",
    0xF5: "u8 payload; expression-indexed dword table lookup",
    0xF6: "u8 payload; dword table lookup",
    0xFF: "end expression / return stack top",
}


def label_name(offset: int) -> str:
    return f"loc_{offset:08X}"


def parse_label_name(name: str) -> int | None:
    if not name.startswith("loc_"):
        return None
    try:
        return int(name[4:], 16)
    except ValueError:
        return None
