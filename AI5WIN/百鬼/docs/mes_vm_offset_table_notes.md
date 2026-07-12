# Baigui MES import notes

## Current finding

2026-07-11 correction: an intermediate Rust importer also scanned every
non-string byte for `F3`, `EF FF`, and `0A`, then treated following operands as
script offsets. EXE `sub_408FD0` proves `F3 imm16` indexes the VM variable table;
it is not a jump. That heuristic changed 12,926 operands in the translated set
and produced the startup error `「・」は存在しません。`. The importer now only
relocates the confirmed leading entry table. VM operands remain unchanged until
their containing instructions are structurally modeled.

The `mes_chs.arc` startup crash is not a cache issue in the ARC layer. Several
MES files begin with a DWORD entry table:

```text
u32 count
u32 entry_offset[count]
...
script/data stream
```

Example: `ADVICE.MES` starts with `count = 0x52`, followed by 82 offsets such as
`0x02DD, 0x0357, 0x0441, 0x053F...`.

The old `mestool` rebuilt the plain MES stream with variable-length translated
strings but preserved these leading DWORD offsets as opaque `.byte` data. After
text length changes, many entry offsets still point to the original positions,
and some can point outside the shorter rebuilt file.

## EXE evidence

The exported decompile confirms the stream is mixed VM/text data:

- `sub_425870`: main VM dispatcher. It evaluates an opcode expression through
  `sub_401E33` / `sub_408FD0`, then reads an argument list through
  `sub_403486` / `sub_40ADA0`.
- `sub_408FD0`: expression decoder. `0xF1` reads a 16-bit immediate, `0xF2`
  reads a 32-bit immediate, and expressions terminate on `0xFF`.
- `sub_40ADA0`: argument list parser. `0x01` means string argument and `0x02`
  means expression argument.
- `sub_425610`: text stream dispatcher. Text display also uses
  `0x01 <cp932 bytes> 0x00`.

Because VM argument strings and text display strings share the same byte shape,
a blind whole-file scan for `0x01 ... 0x00` is not enough to describe the VM.

## Tool changes

- `mestool/disassembler.py`
  - Detects the leading MES entry table.
  - Emits `.entry_table` / `.entry_table_append` metadata into asm.
  - Starts cstring scanning at the script start when an entry table exists, so
    table/header bytes are not exported as text.
  - Emits forced labels at entry offsets when they fall inside raw byte ranges.

- `mestool/assembler.py`
  - Parses `.entry_table` metadata.
  - Maintains an original-offset to rebuilt-offset map while assembling.
  - Rewrites the leading entry table after translated strings change length.
  - Handles entry offsets that land inside a cstring by mapping the original byte
    position to the corresponding rebuilt byte position.

- `mestool/opcode.py`
  - Preserves CP932 vendor-extension bytes in the `0xFA-0xFC` lead-byte ranges
    with `{{FA:AA}}`-style byte placeholders. Python's codec can normalize
    those pairs to different but equivalent code points, so preserving the raw
    bytes is required for byte-exact roundtrip.

## Generated outputs

- `mes_chs_reloc.arc`: repair of the old `mes_chs.arc` by updating only leading
  entry tables.
- `mes_chs_rebuilt.arc`: rebuilt from original `mes.arc` plus JSON files in
  `new/`, using the fixed `mestool` relocation path.

Current deployed test package:

```text
E:\GAL\gui\mes_chs_rebuilt.arc
E:\GAL\gui\AI5FIX.ini -> ArcMesName=mes_chs_rebuilt.arc
```

Static validation for `mes_chs_rebuilt.arc`:

```text
files: 2533
source JSON files: 2533
original non-empty entry tables: 68
original entry table entries: 1792
rebuilt entry table entries checked: 1792
entry count mismatches: 0
rebuilt invalid entry tables: 0
entry offset changed files: 65
plain bytes changed vs mes_chs.arc: 75 files
plain length changed vs mes_chs.arc: 1 file
pack -> unpack byte diffs: 0
sha256: 8DF78C6D4D85F1BE54BBEAF034075993C862AB025FC022EED33C67C67B5B6550
size: 3275129
```

Report:

```text
H:\IDA-PRO-MCP\百鬼\_mes_compare\mes_chs_rebuilt_report.json
```
