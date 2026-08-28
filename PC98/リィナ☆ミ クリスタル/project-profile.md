# Liena Crystal SDT profile

## Ownership

- Game: `Liena Crystal` (PC-98)
- Source role: original game directory, read-only
- Work role: Rust source, generated translation JSON, and rebuilt SDT files
- Delivery role: unknown; do not sync outside the work directory without approval
- Tool language: Rust

## Script format

- Extension: `.SDT`
- Outer archive: none for the analyzed files
- Structure: bytecode stream with NUL-terminated mixed-text operands and 16-bit
  little-endian absolute control-flow targets
- Ordinary double-byte text: CP932 bytes with each byte XOR `0x0A`
- Compressed kana: `0x7E` selects JIS row `0x24`, `0x7F` selects row `0x25`,
  followed by one-byte cells in `0x21..0x7C`
- Gaiji: `0x7D` plus ASCII digit `0..9`; preserve as `<g0>` through `<g9>`
- Runtime wrapping: 29 full-width cells, or 30 for prohibited line-start
  punctuation; three display lines before automatic paging. Runtime wrapping is
  not stored in SDT.
- Confirmed text sources: mixed text operands of opcodes `0x10` and `0x18`, and
  labels inside the structured `0x30` choice block. NUL strings used by other
  opcodes are resource or control parameters and are not extracted.
- Confirmed physical instruction set: `10, 11, 12, 13, 18, 1B, 20, 21, 22, 30,
  40, 41, 42, 45, 50, 51, 52, 53, 54, 80, 81, 82, 84, 85, 87, 8A, 8B, A0, A1,
  B0, B1, B2, BA, BB, BC, BD, BF, E0, E1, E2, F0, FA`. Any other opcode is
  rejected.
- `0x20` is an absolute jump, `0x21` is an absolute call, and `0x22` returns.
  `0x50` and `0x51` are conditional jump/call forms. Conditions are four-byte
  atoms joined by byte `2` and terminated by join byte `0`.

## Text controls

- `<cN>` / `<CN>`: color selection, `N=0..9`
- `<wN>` / `<WN>`: timed wait, `N=0..9`
- `<rN>` / `<RN>`: explicit line break with indentation, `N=0..9`
- `<p>` / `<P>`: page wait
- `<gN>`: one of ten runtime-loaded 16x16 gaiji slots; exact artwork is
  intentionally preserved rather than normalized to Unicode
- NUL is the text operand terminator and is never editable text
- A non-digit raw parameter accepted by the runtime is represented as
  `<c#HH>`, `<w#HH>`, or `<r#HH>` and can be written back byte-exactly. The
  observed `0x2F` wait parameter is therefore `<w#2F>`.

Controls inside `message` are editable, but malformed controls are rejected.
Automatic visual wrapping is neither extracted nor injected.

## Name and message

Named dialogue has the confirmed shape `$`, optional controls, visible
`【name】`, `$`, then message. The delimiters and `$` state transitions are
structural. JSON exposes the bracket contents as writable `name`, keeps the
original in `_scr_name`, and exposes only the following body as `message`.
Narration has no `name`. A literal `【...】` outside the confirmed structure is
ordinary message text.

The user explicitly confirmed that existing `name` fields may be translated
and injected. Names cannot be added to narration or removed from named entries.

## Choices

Opcode `0x30` contains a structured choice list. Choice labels use only
double-byte CP932-XOR text. The runtime copies each encoded label into a
41-byte slot, so payloads are limited to 40 bytes plus NUL.

## Rebuilding

- Unknown non-text bytes are preserved.
- Changed lengths require every affected absolute target to be remapped.
- Rebuilding stops on an unknown opcode, invalid target, malformed control,
  NUL, real CR/LF, or an unencodable character.
- Unchanged extract/inject must reproduce every source byte exactly.
- Injection applies the built-in `subs_cn_jp.json` character substitutions only
  to edited `name` and `message` fields before CP932 encoding. Validation fields
  and unedited text remain unchanged; the table is embedded in the executable.
- The known `22 20 20 1F` sequences immediately following returns in one script
  are unreachable opaque data, not jumps, and remain byte-preserved.

## Verification

- All 10 analyzed SDT files parse sequentially to EOF under the confirmed
  instruction grammar.
- Extraction yields 2771 records: 2592 opcode-`0x10` messages, 117
  opcode-`0x18` messages, and 62 choice labels.
- Extract followed by unchanged inject is byte-exact for all 10 files.
- A real-script modified round trip covered a longer message with edited
  controls, a writable speaker name, a shorter choice, target relocation, and
  successful re-extraction.
