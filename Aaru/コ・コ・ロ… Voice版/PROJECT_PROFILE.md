# KOKOROV FL2 / AB project profile

## Scope and paths

* Project: KOKOROV / Kokona sample, runtime `KOKOROV.EXE`.
* Sample work directory: `E:\GAL\kokona\work`.
* Sample archive: `E:\GAL\kokona\work\A.FL2`.
* Script sample directory: `E:\GAL\kokona\work\mes`.
* Development tool: `H:\IDA-PRO-MCP\kokorov_fl2_tool`.
* Source archive and scripts are immutable. Extraction/injection outputs use
  new paths; formal tool-directory synchronization requires an explicit user
  update.

## FL2 archive

* Magic is `FL2.0\0`.
* Little-endian fields are `u16 header_size` at `0x06`, `u32 file_count` at
  `0x08`, `u32 index_size` at `0x0c`, and `u32 index_offset` at `0x10`.
* `A.FL2` has a 32-byte header and 431 entries. Payloads start at
  `header_size`, are contiguous in entry order, and end exactly at
  `index_offset`.
* Each index record is `u32 payload_size + u8 name_length + name_length CP932`
  bytes; the supplied sample ends with four `0xff` bytes. No compression or
  per-entry transform is applied by the unpacker.
* Header bytes without a confirmed meaning are preserved in the manifest.
* `fl2_pack` rebuilds payload sizes, the full index, `index_size`, and
  `index_offset` from `fl2_manifest.json`. Entry order and names are immutable;
  payload lengths may change. Unknown header bytes are preserved.

## AB script format

* `.AB` is a little-endian opcode stream. The parser follows the interpreter's
  known operand layouts, records every instruction boundary, string, voice id,
  and absolute target, and rebuilds all affected target offsets after length
  changes. Unknown opcodes are rejected.
* Text and voice strings are CP932 and must pass byte-exact decode/encode
  round-trip checks.
* Opcode `0`: `name\Nbody`. Empty `name` is `monologue`. A named body starting
  with `「` or `《` is `dialogue`; other named bodies are confirmed ending/system
  text. The separator belongs to the instruction structure, not `message`.
* Opcode `1`: `choice`, with the source leading fullwidth space preserved.
* Opcode `8`: message-buffer assignment, extracted as `system` with
  `_buffer_index`; the runtime limit is 16 CP932 bytes. In the sample all 40
  values use buffer `M0` and are `僕`, `ぼく`, `ボク`, `俺`, or `オレ`.

## Translation contract

* Exchange format is UTF-8 JSON, one array per source `.AB` file. `scr_msg` is
  immutable source text; `message` is the only writable body field.
* Writable names use `name` plus `_scr_name`; injection verifies `_scr_name`
  against the source before writing `name`. Name presence cannot be changed.
* `_file`, `_index`, `_offset`, `_inst_offset`, `_size`, `_type`, `_opcode`,
  `_target`, `_message_id`, `_voice`, `_buffer_index`, `_encoding`, and `_policy`
  are stable validation metadata. `_index` is scoped to one source script.
* `\N` is the opcode-0 separator and is structural. `\W` and `%M0` are
  exposed controls: the user confirmed that they may be moved, deleted, or
  added across translated entries. Injection validates their syntax but does
  not compare their counts with the source. Real CR/LF, NUL, malformed
  controls, and characters not encodable in CP932 are rejected.
* The policy is `relocate`: strings may become longer or shorter, and every
  parsed absolute target is remapped by `rebuild_script`.

## Verification record

* Unit tests: 31 passing, including byte-exact FL2 packing, changed payload
  sizes and offsets, control removal/repositioning/addition, malformed
  boundaries, CP932 failures, names, choices, opcode 8, script relocation, and
  directory copy/injection behavior.
* Real sample: 238 `.AB` files and 30,599 extractable entries. Unchanged
  extraction/injection reported `patched=0`, `unchanged=30599`; all 238 output
  paths and SHA-256 hashes matched the source tree.
* Modification loop changed an opcode-8 value, a name, short/long bodies, a
  choice, and moved `\W`; re-extraction confirmed the expected values. Only
  `ARS0000.AB` and `AA0_000.AB` differed from the original scripts.
* Runtime display verification is intentionally user-owned and was not run by
  this tool update.
* Real archive: `A.FL2 -> unpack -> unchanged pack` rebuilt 431 entries and
  27,538,085 bytes. Original and rebuilt SHA-256 were both
  `9BD4061B338AB8E8F3D6E1B74D1599899C0D0B390D213925801AF47E0A94E630`.

## Known limitations

* Text outside CP932 requires a separate font/encoding patch.
* Packing requires the unpacker's manifest and cannot add, remove, reorder, or
  rename entries; it only replaces their payload bytes and sizes.
* Unconfirmed opcode variants and unknown instructions are not guessed or
  heuristically scanned.
