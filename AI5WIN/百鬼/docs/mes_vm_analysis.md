# Baigui / AI5WIN MES VM analysis

## Scope

Target: `百鬼 -淫黙された廃墟-` `.MES` scripts.

The script file has two layers:

1. Outer file layer: AI5WIN/Okumura-style LZSS compressed byte stream.
2. Inner plain MES layer: mixed Text/Event stream and VM opcode stream.

The tools in this package operate on both layers:

```text
.MES compressed
  -> LZSS decompress
  -> plain MES semantic assembly / JSON text projection
  -> rebuild plain MES
  -> LZSS recompress
```

## LZSS layer

Constants:

```text
N        = 4096
F        = 18
INIT_POS = 0xFEE
flag bit = 1 literal, 0 backreference
pos      = lo | ((hi & 0xF0) << 4)
length   = (hi & 0x0F) + 3
```

The recompressor is compatible with the decompressor, but it does not attempt to reproduce the vendor-compressed byte stream exactly.  Validation must compare the decompressed plain MES bytes.

## Text/Event stream

Main dispatcher:

```text
sub_40272A -> sub_425610
```

The stream reader is vtable `+0x14`:

```text
mode 0 = peek next byte
mode 1 = read/advance next byte
```

Text control bytes confirmed:

| byte | meaning |
|---:|---|
| `0x00` | text/end terminator in control dispatcher |
| `0x01` | inline CP932/SJIS c-string text segment: `01 <bytes> 00` |
| `0x0B` | text display item boundary / commit-and-wait / block boundary |
| `0x10` | parameter/time-like control path; calls argument parser before target callback |
| `0x11` | flag/condition-like control path |

SJIS/CP932 double-byte lead ranges:

```text
0x81-0x9F
0xE0-0xEF
0xFA-0xFC
```

### `0x01 <cstring>`

`0x01` introduces a text segment.  The renderer consumes following CP932/SJIS bytes until the terminating `0x00` or another non-text control boundary.

Confirmed examples from `P_01R2.MES` plain stream:

```text
01 97 B3 88 EA 00                -> 竜一
01 8D 82 8E 75 00                -> 高志
01 81 75 ... 81 76 00            -> 「...」
```

### `0x0B`

`0x0B` is not a simple name/message separator.  It marks a text display unit boundary and participates in commit/wait behavior.  In dialogue blocks, it is a reliable anchor for grouping strings:

```text
0B ... 01 <name> 00 ... 01 <message> 00
0B ... 01 <message> 00
```

Therefore extraction groups by `0x0B` blocks:

- two display strings: first short non-quoted string = `name`, second = `message`.
- one display string: monologue / narration.

## TextBlock manager

Function:

```text
sub_427A70
```

It manages a 64-entry text block ring buffer.  It does not directly parse script bytes.

```c
struct TextBlock {
    char     text[0x800];
    uint32_t field_0;
    uint32_t status;
    uint32_t value;
};
```

Layout:

```text
base          = this + 0x3350
block_size    = 0x80C
block_count   = 64
current_index = this + 0x23650
```

`sub_427A70` cases:

| case | meaning |
|---:|---|
| 0 | reset all blocks |
| 1 | open/start current block |
| 2 | commit/close current block |
| 3 | count active blocks |
| 4 | get block text pointer |
| 5 | get block value |

## Real-time text renderer

Function:

```text
sub_416450
```

It directly consumes CP932/SJIS bytes from the script stream and renders glyphs.  It does not write to the TextBlock history buffer.

Core pipeline:

```text
read lead/trail bytes
  -> combine sjis = (lead << 8) | trail
  -> sub_40B9F0 glyph/cursor handling
  -> sub_417A20 glyph lookup
  -> sub_417930 glyph blit
  -> sub_410450 / sub_40F580 layout
  -> sub_407500 final output blit
```

Known rendering constants:

```text
glyph cell = 24 x 24
font charset = SHIFTJIS_CHARSET / 0x80
```

## VM opcode stream

VM dispatch:

```text
sub_425870
```

Opcode value is read through the expression decoder, not as a raw byte in every case.

Expression decoder:

```text
sub_401E33 -> sub_408FD0
terminated by 0xFF
```

Argument parser:

```text
sub_403486 -> sub_40ADA0
```

Argument tags:

| tag | meaning |
|---:|---|
| `0x00` | end of argument list |
| `0x01` | string argument, null terminated |
| `0x02` | expression/integer argument, decoded via `sub_408FD0` |

Argument slot layout:

```text
slot_size      = 45
slot[i].type   = this + 9764 + 45*i
slot[i].str    = this + 9765 + 45*i
slot[i].int    = this + 9805 + 45*i
```

Important opcode semantics:

| opcode | current name | notes |
|---:|---|---|
| `0x01` | `TEXT_GATE` / `MSG_STATE` | handler `sub_425AE0 -> vtable+0xBC -> sub_422F10`; state/check/commit, not direct string parse |
| `0x0D` | `CALL_MES_CANDIDATE` | likely MES/subscript call candidate |
| `0x18` | `CHOICE_MENU` | strongest choice/menu candidate, handler `sub_428470` |

## Choice/menu structure

Choice display text is still stored as `0x01 <cp932 cstring> 0x00`, but it is recognized by menu context, not by dialogue block grouping.

Confirmed example from `P_03.MES`:

```text
01 選択肢：見つけた原稿用紙 00    -> choice group label, not exported as message
01 普通に読む 00
01 斜めに読む 00
01 破り捨てる 00
```

The extractor exports the three menu strings as `_type=choice` and stores the label in `_label`.

## JSON projection

Translator-visible fields:

```text
name      optional speaker name, only when present
scr_msg   original text, used for validation
message   translated text to inject
```

Internal fields use `_` prefix.  The injector writes back `message`, and writes back `name` only when `name` exists and `_name_inst_offset` is present.

## Rebuild policy

The assembler rebuilds the plain MES stream from semantic assembly and can then LZSS-compress it.

- Plain roundtrip must be byte exact: `MES -> plain -> asm -> plain`.
- Recompressed `.MES` may differ byte-for-byte from the original compressed file, but must decompress to the rebuilt plain stream.

Unknown VM/control bytes are emitted as `.byte` and preserved exactly.
