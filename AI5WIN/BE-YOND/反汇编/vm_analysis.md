# b_mes VM analysis

## Scope

Input files are already-decompressed AI5WIN `.MES` bytecode. The tool does not apply LZSS.

This build was checked against all 255 files from the supplied `mes.zip`.

## EXE-backed anchors

From `ai5win.exe.c`:

- Main bytecode loop: `sub_403AD0`.
- Expression reader: `sub_40AFC0`.
- Argument-list reader: `sub_403690`.
- Text display handlers: `sub_403E60` for opcode `0x01`, `sub_404060` for opcode `0x02`.
- Jump/call/branch handlers: `sub_4044B0` (`0x09`), `sub_404520` (`0x0A`), `sub_404950` (`0x0E`), `sub_405710` (`0x12`).

## Top-level opcode table

| Opcode | Mnemonic | Format | Handler |
|---:|---|---|---|
| 00 | RETURN | none | sub_403AD0 case 00 |
| 01 | TEXT | cstring | sub_403E60 |
| 02 | SYSTEM_TEXT | cstring | sub_404060 |
| 03 | SET_BYTE_VAR | u16 expr_list_00 | sub_4041A0 |
| 04 | SET_WORD_TABLE | u8 expr_list_00 | sub_404210 |
| 05 | SET_BYTE_BY_EXPR | expr expr_list_00 | sub_404280 |
| 06 | SET_BYTE_TABLE_BASE | expr u8 expr_list_00 | sub_404350 |
| 07 | SET_WORD_TABLE_BASE | expr u8 expr_list_00 | sub_4043C0 |
| 08 | SET_DWORD_TABLE_BASE | expr u8 expr_list_00 | sub_404440 |
| 09 | JUMP_IF_EXPR_NOT_1 | expr target:u32le | sub_4044B0 |
| 0A | JUMP | target:u32le | sub_404520 |
| 0B | SYS | expr arglist_00 | sub_404580 |
| 0C | SET_CONTEXT_STRING | arglist_00 | sub_404710 |
| 0D | MENU_ACCUM | arglist_00 | sub_404770 |
| 0E | REGISTER_BRANCH_AND_SKIP | arglist_00 target:u32le | sub_404950 |
| 0F | CALL_SAVE_CONTEXT | none | sub_403690 + sub_404A70 |
| 10 | DISPLAY_CONTROL | arglist_00 | sub_404AC0 |
| 11 | LINE_ADVANCE | u8 | sub_4056E0 |
| 12 | CALL_SAVE_JUMP | expr target:u32le | sub_405710 |
| 13 | WAIT_COMMIT | none | sub_405790 |
| 14 | SET_DWORD_TABLE | u8 expr_list_00 | sub_4042E0 |

Bytes not safely recognized as one of the above are emitted as `.byte` so that tables or non-executed binary regions remain byte-exact.

## Expression grammar

`sub_40AFC0` reads one or more expression tokens until `0xFF`.

| Token | Payload | Meaning summary |
|---:|---:|---|
| 80 | u8 | byte/word variable lookup |
| A0 | u8 | expression-indexed word lookup |
| C0 | u8 | expression-indexed byte lookup |
| E0..F0 | none | runtime helper token |
| F1 | u16le | immediate word |
| F2 | u32le | immediate dword |
| F3 | u16le | byte source address/index |
| F4 | none | expression-indexed byte source |
| F5 | u8 | expression-indexed dword lookup |
| F6 | u8 | dword table lookup |
| FF | none | expression terminator |
| other | none | small literal/default token |

## Argument list grammar

`sub_403690` parses a zero-terminated list:

- `00`: end of argument list.
- `01 <cstring> 00`: string argument.
- `02 <expr>`: expression argument.

## Jump label policy

`0x09`, `0x0A`, `0x0E`, and `0x12` use absolute `u32le` bytecode offsets. The disassembler emits `loc_XXXXXXXX`; the assembler resolves labels back to offsets, so later relocation work can be built on this representation.

## Known limitations

- Some `.MES` files such as `STAND.MES`, `STAND2.MES`, `NAME.MES`, and `0.MES` contain substantial table/binary data. These are preserved as `.byte` records where the byte stream cannot be safely interpreted as VM instructions.
- The current tool is intended for zero-mutation disassembly/reassembly and VM-boundary validation. It is not yet a JSON text extractor/injector.
- `SYS` subcommand semantics are not expanded individually; their argument bytes are preserved through `{{..}}` placeholders.
