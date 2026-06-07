# -*- coding: utf-8 -*-
"""AI5WIN b_mes opcode model.

Grounded in the uploaded ai5win.exe.c decompile:
- main VM loop: sub_403AD0
- expression reader: sub_40AFC0
- argument-list reader: sub_403690

The assembler/disassembler are intentionally conservative: confirmed script
commands are decoded semantically; any byte sequence that is not safe to split is
kept as .byte records, so zero-mutation rebuild remains possible.
"""
from __future__ import annotations

DEFAULT_ENCODING = "cp932"

TEXT_LEAD_RANGES = ((0x81, 0x9F), (0xE0, 0xEF), (0xFA, 0xFC))

def is_sjis_lead(b: int) -> bool:
    return any(lo <= b <= hi for lo, hi in TEXT_LEAD_RANGES)

def label_name(offset: int) -> str:
    return f"loc_{offset:08X}"

def parse_label_name(name: str) -> int | None:
    if not name.startswith("loc_"):
        return None
    try:
        return int(name[4:], 16)
    except ValueError:
        return None

# Expression token lengths inside sub_40AFC0, including token byte itself.
# Tokens not listed consume only themselves. 0xFF terminates an expression.
EXPR_TOKEN_PAYLOAD = {
    0x80: 1,
    0xA0: 1,
    0xC0: 1,
    0xF1: 2,
    0xF2: 4,
    0xF3: 2,
    0xF5: 1,
    0xF6: 1,
}

OPCODES = {
    0x00: {"mnemonic": "RETURN", "format": "none", "handler": "sub_403AD0:case_00"},
    0x01: {"mnemonic": "TEXT", "format": "cstring", "handler": "sub_403E60"},
    0x02: {"mnemonic": "SYSTEM_TEXT", "format": "cstring", "handler": "sub_404060"},
    0x03: {"mnemonic": "SET_BYTE_VAR", "format": "u16 expr_list_00", "handler": "sub_4041A0"},
    0x04: {"mnemonic": "SET_WORD_TABLE", "format": "u8 expr_list_00", "handler": "sub_404210"},
    0x05: {"mnemonic": "SET_BYTE_BY_EXPR", "format": "expr expr_list_00", "handler": "sub_404280"},
    0x06: {"mnemonic": "SET_BYTE_TABLE_BASE", "format": "expr u8 expr_list_00", "handler": "sub_404350"},
    0x07: {"mnemonic": "SET_WORD_TABLE_BASE", "format": "expr u8 expr_list_00", "handler": "sub_4043C0"},
    0x08: {"mnemonic": "SET_DWORD_TABLE_BASE", "format": "expr u8 expr_list_00", "handler": "sub_404440"},
    0x09: {"mnemonic": "JUMP_IF_EXPR_NOT_1", "format": "expr target:u32le", "handler": "sub_4044B0"},
    0x0A: {"mnemonic": "JUMP", "format": "target:u32le", "handler": "sub_404520"},
    0x0B: {"mnemonic": "SYS", "format": "expr arglist_00", "handler": "sub_404580"},
    0x0C: {"mnemonic": "SET_CONTEXT_STRING", "format": "arglist_00", "handler": "sub_404710"},
    0x0D: {"mnemonic": "MENU_ACCUM", "format": "arglist_00", "handler": "sub_404770"},
    0x0E: {"mnemonic": "REGISTER_BRANCH_AND_SKIP", "format": "arglist_00 target:u32le", "handler": "sub_404950"},
    0x0F: {"mnemonic": "CALL_SAVE_CONTEXT", "format": "none", "handler": "sub_403690 + sub_404A70"},
    0x10: {"mnemonic": "DISPLAY_CONTROL", "format": "arglist_00", "handler": "sub_404AC0"},
    0x11: {"mnemonic": "LINE_ADVANCE", "format": "u8", "handler": "sub_4056E0"},
    0x12: {"mnemonic": "CALL_SAVE_JUMP", "format": "expr target:u32le", "handler": "sub_405710"},
    0x13: {"mnemonic": "WAIT_COMMIT", "format": "none", "handler": "sub_405790"},
    0x14: {"mnemonic": "SET_DWORD_TABLE", "format": "u8 expr_list_00", "handler": "sub_4042E0"},
}

MNEMONIC_TO_OPCODE = {v["mnemonic"]: k for k, v in OPCODES.items()}
