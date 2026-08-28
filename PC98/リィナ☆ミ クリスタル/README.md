# Liena Crystal SDT text tool

`liena-sdt` extracts and injects the structured `.SDT` scripts used by the
PC-98 game *Liena Crystal*. It parses the bytecode instruction stream rather
than scanning for text-like byte sequences, and it rebuilds 16-bit absolute
jump and call targets when a translation changes length.

## Build

Install a current Rust toolchain, then run:

```console
cargo build --release --offline
```

The executable is written to `target/release/liena-sdt.exe`. Run it without
arguments for the interactive menu. Passing one path pre-fills the first path
prompt; no files are written until the settings are confirmed.

## Commands

Extract one UTF-8 JSON file per SDT:

```console
liena-sdt extract --source <GAME_DIR> --output <JSON_DIR>
```

Inject translations into a complete copy of the source tree:

```console
liena-sdt inject --source <GAME_DIR> --translations <JSON_DIR> --output <OUTPUT_DIR>
```

Check that untouched JSON rebuilds every translated SDT byte-for-byte:

```console
liena-sdt verify --source <GAME_DIR> --translations <JSON_DIR>
```

Existing output directories are rejected unless `--overwrite` is supplied.
Output is prepared in a sibling staging directory before it replaces the final
directory. The output path may not overlap the source, translation, or mapping
input path.

All commands accept `--mapping <CHARACTER_MAP.json>`. This optional file maps
Unicode characters directly to patched font byte codes and is separate from the
built-in translation substitutions described below. Use `--help` or
`<COMMAND> --help` for the complete argument list.

## Built-in Chinese-to-game character substitutions

During injection, the tool automatically applies the table embedded from
`subs_cn_jp.json` to edited `name` and `message` fields before encoding them.
Source-validation fields such as `scr_msg` and `_scr_name` are never changed.
Unedited fields are left byte-exact, so the table does not alter an unchanged
round trip. The injection summary reports how many character occurrences were
mapped.

The table is compiled into the executable; it does not need to be copied beside
the release binary. Rebuild the executable after editing `subs_cn_jp.json`.

## Translation JSON

Edit only these fields:

- `message`: dialogue, narration, or a choice label written during injection.
- `name`: the writable speaker name on entries where it is present.

Validation fields must remain unchanged:

- `scr_msg`: original message used to identify and validate the source entry.
- `_scr_name`: original speaker name used to validate a writable `name`.
- `_file`, `_index`, `_offset`, `_size`, `_type`, and `_opcode`: source
  location and structure metadata.

The tool removes the structural `【` and `】` around a recognized speaker name
from the editable fields and restores them automatically. A `【...】` sequence
in ordinary narration remains part of `message`.

## Text controls

Controls inside `message` may be added, removed, reordered, or changed:

- `<cN>` / `<CN>`: color, where `N` is `0..9`.
- `<wN>` / `<WN>`: timed wait.
- `<rN>` / `<RN>`: explicit line break and indentation.
- `<p>` / `<P>`: page wait.
- `<gN>`: runtime gaiji slot `0..9`; keep these tokens unless the matching
  glyph resource is intentionally changed.
- `<$>`: text-state control. The two controls that delimit a recognized name
  are structural and are managed by the tool.

Rare source parameters outside `0..9` are represented losslessly as
`<c#HH>`, `<w#HH>`, or `<r#HH>`, where `HH` is the raw hexadecimal byte.
Malformed controls, NUL, and real CR/LF are rejected.

The game wraps text automatically at 29 full-width cells, with a limited
30-cell punctuation exception, and automatically pages after three display
lines. Those visual wraps are not stored in SDT and are not added to JSON.
Only explicit `<rN>` and `<p>` controls are extracted.

## Encoding and limits

Normal glyphs use two CP932 bytes, each XORed with `0x0A`. The source format
also has compressed kana and ten gaiji slots; unchanged entries retain their
original bytes exactly. Changed entries are encoded as full two-byte glyphs.

After the built-in substitutions run, characters still outside CP932 require a
character map that matches the game's font patch. The map values are the two
decoded bytes before XOR:

```json
{
  "characters": {
    "你": "FA40"
  }
}
```

Choice labels support only two-byte glyphs and may occupy at most 40 encoded
bytes. A rebuilt SDT may not exceed 65535 bytes. Injection stops on malformed
bytecode, invalid control-flow targets, unsupported characters, stale JSON
metadata, or an output file that is locked by another program.
