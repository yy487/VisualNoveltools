# Tauhido localization tool

`tauhido_nfd_unpacker` handles the complete Tauhido PC-98 localization path:
validated NFD R0/N88 extraction, structured `DISK-A` and `DISK-B` script parsing,
`AG00` command-label extraction, UTF-8 translation JSON, pointer-aware injection,
`font.tmp` regeneration, and NFD rebuilding. Inputs are never modified.

## Build

Use a recent stable Rust toolchain on Windows. Font rendering uses Windows GDI.

```powershell
cargo build --release --offline --bins
```

The executable is written to `target/release/tauhido_nfd_unpacker.exe`.

## Localization workflow

Extract all three original images into one managed workspace:

```powershell
tauhido_nfd_unpacker.exe extract-localization --input <GAME_DIR> --output <WORKSPACE>
```

Edit only `message` in the JSON files under `translation_json/`. Keep `scr_msg`
and all underscore-prefixed location fields unchanged. Files are UTF-8 JSON;
there is no TXT or DSAT translation path.

Build the translated images and matching font:

```powershell
tauhido_nfd_unpacker.exe pack-localization --input <GAME_DIR> --workspace <WORKSPACE> --output <OUTPUT_DIR>
```

The rebuild directory contains all three NFD images, `font.tmp`, and
`rebuild_manifest.json`. The manifest records hashes, replaced members, the
resolved Unicode/carrier map, and every redrawn slot. Copy `font.tmp` together
with the rebuilt disks when testing or distributing the patch.

`--input` accepts an image or a directory whose immediate files are detected by
signature. It can be repeated. Existing managed output can be replaced with
`--overwrite`; when refreshing a localization workspace, translations whose
source file, source hash, offset, type, and `scr_msg` still match are preserved.
Newly discovered entries retain their Japanese source text. Unrelated non-empty
directories are refused.

## Text and control behavior

`DISK-A` and `DISK-B` are parsed as 256-byte records with a record pointer table,
page map, per-script command table, reachable bytecode paths, local jumps,
choices, expressions, and text spans. Variable-length injection rewrites local
pointers and file-level record pointers.

The engine does not perform horizontal word wrapping. A line has 40 fullwidth
columns. `R` starts the next line at X=1, while `B` starts it at X=9. Script
controls (`A`, `B`, `F`, `G`, `L`, `M`, `P`, `Q`, `R`, `S`, `U`, `X`, `Y`, `Z`,
`!`, `&`, `@`, `$`, `[`, `:`, `]`, `{`, `}`) are structural and are not embedded
in editable `message` strings. `]` returns from the engine's input-code handler
and does not terminate the surrounding script path. A `{...}` conditional has
two reachable successors: its body when the condition succeeds and the byte
after the matching `}` when it fails. Nested conditionals are matched
structurally so text on either path is extracted.

`AG00` is not scenario bytecode. It has an ASCII count header and verb/object
records containing 7-bit JIS pairs between `ESC K` and `ESC H`. Its first object
`*` is structural and cannot be translated. Although its storage codec differs,
it uses the same PC-98 glyph slots as the scenario scripts.

ASCII graphic characters in translations are normalized to fullwidth forms.
The embedded `subs_cn_jp.json` supplies preferred Chinese-to-CP932 carrier slots;
collisions are resolved deterministically. Every carrier slot used by the final
`DISK-A`, `DISK-B`, and `AG00` text is redrawn with the same 16×16 Windows font,
including directly encodable Japanese characters. `NACT8S` is intentionally not
extracted, injected, scanned, or protected from slot replacement; the archived
`NACT8S.B` member is likewise outside the localization scope.

## Archive-only extraction

The original validated unpack mode remains available:

```powershell
tauhido_nfd_unpacker.exe unpack --input <GAME_DIR> --output <OUTPUT_DIR>
```

Running without arguments, or passing paths without a command, opens the full
interactive menu for archive extraction, localization extraction, and
localization packing. A fully specified command is non-interactive.

## Validation and limits

The NFD parser requires R0 images with one unambiguous N88 filesystem, three
identical FAT copies, valid directory entries, complete 26-sector data tracks,
and disjoint acyclic cluster chains. Rebuilding updates all FAT copies, directory
start clusters, member sector data, and any newly allocated chains. Unknown NFD
sectors and unrelated members remain byte-preserved.

NFD R1 and unrelated N88 layouts are rejected. Unknown nonzero script-tail data
normally keeps its original script-relative offset. If translated text grows
across such data, the tool relocates it only when it follows a `0x1A` script
terminator and no parsed pointer targets the region; otherwise packing stops.
Glyph generation requires Windows and a usable `新宋体` face.
