# KOKOROV FL2 / AB localization tool

This Rust tool handles the confirmed KOKOROV sample format. In the examples
below, `<GAME_DIR>` is the game data directory and `<OUTPUT_DIR>` is a separate
output directory:

* `fl2_unpack` extracts the `FL2.0\0` archive without modifying it.
* `fl2_pack` rebuilds an archive from an unpacked directory and its manifest.
* `ab_inspect` validates `.AB` bytecode and prints opcode statistics.
* `ab_extract` writes one UTF-8 JSON translation file per `.AB` script.
* `ab_inject` validates JSON and rebuilds translated `.AB` scripts with
  relocated absolute targets.

The source archive and source scripts are never overwritten by default.

## Build

From this directory:

```powershell
cargo fmt -- --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --release --offline --bins
```

The release binaries are in `target\release`.

## Archive usage

```powershell
fl2_unpack.exe "<GAME_DIR>\A.FL2"
fl2_unpack.exe --output "<OUTPUT_DIR>\A_unpacked" "<GAME_DIR>\A.FL2"
fl2_pack.exe --output "<OUTPUT_DIR>\A_packed.FL2" "<OUTPUT_DIR>\A_unpacked"
```

The default output is `<stem>_unpacked`. Existing output is rejected unless
`--overwrite` is supplied. `fl2_manifest.json` records the header, index,
entry order, CP932 name bytes, payload offsets, sizes, and output paths.

`fl2_pack` requires that manifest. It validates the preserved header, original
entry order, CP932 names, index terminator, and safe paths. It reads each
manifest-listed payload from the unpacked directory, permits payload sizes to
change, recalculates every entry size plus `index_size` and `index_offset`, and
preserves unknown header bytes. Extra files not listed in the manifest are not
packed. Existing output is rejected unless `--overwrite` is supplied.

To install injected scripts into an archive, unpack `A.FL2`, copy the contents
of `mes_injected` over the matching `.AB` files in `A_unpacked`, and then run
`fl2_pack`. The original `A.FL2` is never overwritten by default.

## AB extraction and injection

Single-file workflow:

```powershell
ab_extract.exe "<GAME_DIR>\mes\ARS0000.AB" --output "<OUTPUT_DIR>\ARS0000.AB.json"
ab_inject.exe "<GAME_DIR>\mes\ARS0000.AB" `
  "<OUTPUT_DIR>\ARS0000.AB.json" --output "ARS0000_injected.AB"
```

Directory workflow:

```powershell
ab_extract.exe "<GAME_DIR>\mes" --output "mes_json"
ab_inject.exe "<GAME_DIR>\mes" "mes_json" --output "mes_injected"
```

Directory extraction scans `.AB` files recursively and writes matching
`relative\path.AB.json` files. Directory injection first validates every JSON
against its matching source script, copies the complete source tree (including
non-AB files), and then replaces only translated scripts. `--overwrite` is
required to replace an existing output. `--output` accepts one path and works
with Windows drag-and-drop paths containing spaces or non-ASCII characters.

## JSON contract

Each JSON file is an array. Extraction initializes `message == scr_msg`.
`scr_msg` is immutable source text and is never written back. `_file` uses a
forward-slash relative path for directory extraction; `_index` is the stable
extractable-text index within that script. Other metadata (`_offset`,
`_inst_offset`, `_size`, `_type`, `_opcode`, `_target`, `_message_id`, `_voice`,
`_buffer_index`, `_encoding`, `_policy`) is validated before injection.

For a writable opcode-0 speaker name, `name` is the editable value and
`_scr_name` is the original value used for validation. Name translation is
allowed only when the source had a name. The current target encoding is CP932;
unencodable characters, NUL, real CR/LF, invalid metadata, and malformed
controls are rejected with a non-zero exit status.

## Confirmed AB text rules

* Opcode `0` stores `name\Nbody`. A non-empty name with a body beginning in
  `「` or `《` is `dialogue`; a non-empty name with another body is `system`
  (the ending title/body form); an empty name is `monologue`.
* Opcode `1` is `choice`; its leading fullwidth space is preserved.
* Opcode `8` is extracted as `system` with `_buffer_index`. The sample uses
  buffer `M0` for `僕`, `ぼく`, `ボク`, `俺`, and `オレ`; the runtime buffer value
  is limited to 16 CP932 bytes.
* `\N` is the opcode-0 name/body separator and is not accepted inside a
  translated body. `\W` and `%M0` are runtime controls exposed in `message`.
  They may be moved, deleted, or added; injection validates their syntax but
  does not compare their counts with the source. `%M0` expands the value
  written by opcode 8; `\W` is a runtime pause.
* Rebuilds are structure-aware: every parsed instruction and absolute target is
  retained, and variable-length strings relocate target offsets. Unknown
  opcodes are rejected rather than guessed.

## Verification

The current sample run produced 238 JSON files and 30,599 entries (30,410
opcode-0 messages, 149 choices, and 40 opcode-8 buffers). The unchanged
`mes -> JSON -> inject` loop reported `patched=0`, `unchanged=30599`; all 238
relative paths and SHA-256 hashes matched the source tree. A modification loop
covered an opcode-8 value, a writable name, shorter and longer messages, a
  choice, and moved `\W`; re-extraction confirmed the changes and only two
scripts differed from the source.

The archive loop `A.FL2 -> unpack -> unchanged pack` rebuilt all 431 entries.
The 27,538,085-byte output had the same SHA-256 as the original archive, proving
a byte-exact no-change round trip. Unit tests also cover changed payload sizes
and rebuilt entry offsets.

## Limitations

Only the confirmed KOKOROV FL2.0 and `.AB` opcode layouts are supported.
Translation text must be representable in CP932; font and encoding patches for
additional characters are outside this tool. Packing requires the manifest
created by this unpacker and does not add, remove, or rename archive entries.
The parser intentionally fails on an unknown or structurally invalid opcode
instead of silently scanning binary data.
