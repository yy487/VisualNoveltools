# 2nd LOVE nbook Xscript extractor

This read-only tool extracts text from the plain-text `.xbk` source scripts
under `E:\GAL\2nd\nbook\book`. The `.xbx` files are compiled counterparts,
and `.ybk`/`.ybx` are scene-transfer scripts; they are intentionally ignored.

The reverse-engineering reference is
`E:\迅雷云盘\work\2nd love\nbook.exe_export_for_ai`; the working copy and
JSON output are under `E:\GAL\2nd\work`. The parser does not depend on a
machine-specific install path.

## Build

```powershell
cargo fmt -- --check
cargo test --offline
cargo build --release --offline
```

## Extract

```powershell
target\release\nbook_xscript_tool.exe extract `
  --input 'E:\GAL\2nd\nbook\book' `
  --output 'E:\GAL\2nd\work\nbook_json_clean'
```

The output directory must not already exist. There is one UTF-8 JSON file per
`.xbk`, preserving the source directory layout. Each entry contains:

- `name`: cleaned text inside a leading `【...】` name marker, when present;
- `_scr_name`: original name-marker text before control filtering;
- `_scr_msg_raw`: original CP932 body with formatting controls;
- `scr_msg`: immutable body after the confirmed control filtering;
- `message`: initialized to `scr_msg` for a future translation pass;
- `_file`, `_line`, `_offset`, `_inst_offset`, `_size`, `_raw_hex`, `_prefix`;
- `_name_line`, `_name_offset`, and `_name_size` when a name marker is present.

The confirmed filter removes paired `@...@` and `_..._` control tokens,
`*RRGGBB*` color markers, and message-boundary markers `#<>+`. Empty entries
that contain only controls are skipped. The raw body and byte range remain in
`_scr_msg_raw`/`_raw_hex` for source validation.

Representative source forms (CP932, shown decoded):

```text
        【水原　麻由美】<　+
        それにしても……大きくなったわね。藍澤くん。#
        そうにしたって、昨日会った時はそんなコト、全然、言っ@70@@70@あ……#
        【*FFFF00*水原　麻由美（みずはら　まゆみ）*FFFFFF*】先生ですか？#
```

Both standalone and inline forms produce `name`; the text after `】` is used as
the same-line body when it contains non-control characters. A standalone name
is attached to the next non-empty body line, allowing intervening control-only
lines to be skipped. `3th.xbk` lacks the usual nbook header but has the same
`@WIN` structure.

## Format profile (analysis status)

- confirmed structure: `.xbk` is CP932 text with `@WIN { ... }` windows;
  bodies occur on non-empty lines inside those blocks;
- confirmed structure: normal body lines are preceded by CRLF + two tabs
  (`0D0A0909`), with a small number of source indentation exceptions;
- confirmed name/message rule: `【...】` is a name marker; the remaining text
  is the message, including inline name/message lines;
- confirmed control policy: remove non-text `@...@`, `_..._`, `*RRGGBB*`,
  `#<>+` markers from translator-visible fields while preserving raw bytes;
- opaque-preserved: the runtime meaning of the removed controls;
- injection/relocation: not implemented; this tool only reads and emits JSON.

The full sample run produced 9,318 entries, including 4,323 named entries;
20 control-only lines were skipped and 21 structural warnings were reported.
The extractor rejects CP932 decode or byte-round-trip failures instead of
silently replacing characters.
