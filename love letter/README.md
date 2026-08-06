# Love Letter `.o` text tool

This Rust tool extracts and injects translatable scenario text from the
`Love Letter` `.o` VM scripts. Classification is based on decoded bytecode
consumers, not Japanese-character, tag, variable, filename, or path heuristics.
Inputs and existing outputs are never overwritten.

## Reverse-engineered format

The executable evidence is `sub_408DF0` in
`loveletter.exe_export_for_ai/decompile/408DF0.c`:

- Opcodes `0x00`, `0x01`, `0x02`, `0x03`, `0x05`, `0x06`, and `0x07` have a
  four-byte little-endian operand.
- Opcode `0x08` and expression operators `0x80..=0xFF` are one byte.
- Opcode `0x02` evaluates a length-prefixed string record; it does not mean
  dialogue.
- Opcode `0x05` invokes a native command and opcode `0x06` invokes a script
  function. They identify the consumer of strings accumulated by a statement.
- The instruction region is followed by `u32 byte_length + payload` records
  reaching EOF. Every record start has an instruction-boundary `0x02`
  reference.

The parser rejects invalid opcodes, truncated operands/records, non-record
targets, unreferenced records, and strings without a following consumer. The
injector rebuilds the suffix table and updates only decoded `0x02` operands;
unknown record payloads and unrelated instruction bytes remain unchanged.

## Extraction policy

The 47 source objects use two confirmed translatable sinks:

- Script call `0x15C27`: scenario message. `_type` is `dialogue`, or `effect`
  when the internal tag is `!se`.
- Script call `0x1870E`: choice text. Blank full-width choice slots are not
  emitted.

All other consumers are excluded, including UI/status text, title and ending
labels, resource names, audio/image paths, diagnostics, comparisons, and file
I/O operands. In particular, `savefile{{VAR:0111}}.dat` is followed by mode
`2` and native command `0x400`; the executable's `0x400` handler builds
`%s\\save\\%s` and opens the file. It is not dialogue.

This rule also fixes the opposite error in the old extractor: untagged
narration passed to `0x15C27` is now `dialogue`, not `ui`. No `_type: "ui"`
entry is emitted.

## Names and controls

The confirmed split remains:

```text
!m\id\nNAME　「MESSAGE」
```

The internal prefix is preserved in `_scr_raw` and exposed as `_tag`/`_id`.
The body inside the final `」` becomes `scr_msg`/`message`. A protagonist name
stored as byte `0x07` plus four ASCII digits is represented as
`{{VAR:0083}}`; it is retained in genuine names and dialogue.

`sub_408DF0` defines a string mini-language. The tool parses each construct as
one indivisible token:

```text
07 + dddd                         {{VAR:dddd}}
08 + width/reserved/wide + dddd   {{VAR_FMT:dddd:width:reserved:wide}}
09                                {{STACK}}
0A + width/reserved/wide          {{STACK_FMT:width:reserved:wide}}
```

`width` is a signed decimal `i32`; `reserved` and `wide` are eight-digit
hexadecimal `u32` values. Other preserved control bytes use `{{CTRL:hh}}`.
The injector requires the ordered token signature to remain unchanged, so a
variable name cannot silently become literal text and a formatted construct
cannot be split into unrelated bytes.

Names are validation-only by default. `_scr_name` must match the source; use
`--write-names` only for an explicitly reviewed literal name change. Runtime
protagonist variables normally remain unchanged.

## JSON contract

Each `.o` file gets one UTF-8 JSON array. Only `message` is normally edited:

```json
{
  "name": "やよい",
  "scr_msg": "瑞穂……",
  "message": "瑞穂……",
  "_file": "M_01.o",
  "_index": 276,
  "_offset": 156482,
  "_entry_offset": 156478,
  "_inst_offset": 131171,
  "_size": 34,
  "_type": "dialogue",
  "_opcode": "0x02",
  "_encoding": "CP932",
  "_policy": "relocate",
  "_scr_raw": "!m\\yay01000\\nやよい　「瑞穂……」",
  "_scr_name": "やよい",
  "_tag": "m",
  "_id": "yay01000",
  "_split": true,
  "_quoted": true,
  "_terminator_len": 1
}
```

`scr_msg`, `_scr_raw`, `_scr_name`, locations, instruction references, type,
encoding, policy, tags, split/quote fields, size, and terminator length are
validated against the source before any output is written.

## Commands

Run from `H:\IDA-PRO-MCP\loveletter\obj-text-tool`:

```powershell
cargo run --offline -- extract E:\GAL\love\obj --output .\obj-json
cargo run --offline -- inject E:\GAL\love\obj .\obj-json --output .\obj-injected
```

Single files are supported as well:

```powershell
cargo run --offline -- extract E:\GAL\love\obj\M_01.o
cargo run --offline -- inject E:\GAL\love\obj\M_01.o E:\GAL\love\obj\M_01.o.json
```

Directory injection copies all source files and only rebuilds objects with a
matching JSON file. Existing output paths are refused.

## Verification and limits

The semantic implementation was checked against all 47 source files:

- 38,905 instruction-boundary string references and 38,905 table records
  decoded without an invalid opcode or boundary mismatch.
- 21,642 extracted entries: 20,578 dialogue, 990 effects, and 74 nonblank
  choices; zero UI entries and zero warnings.
- Unchanged injection reported 21,642 unchanged entries; all 47 rebuilt files
  were SHA-256 identical to the source.
- Real short and long edits to `M_01.o` re-extracted correctly. All 46
  unrelated files stayed identical, and every changed instruction byte was
  inside a decoded `0x02` operand.

The sink addresses and mini-language are specific to this confirmed
`Love Letter` profile. The tool deliberately rejects a structurally different
VM layout instead of falling back to heuristic text scanning. It does not pack
an outer archive or patch the executable.

Release checks:

```powershell
cargo fmt -- --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --release --offline --bins
cargo run --release --offline -- --help
```
