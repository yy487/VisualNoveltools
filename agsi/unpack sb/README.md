# AGSI SB2 Archive Tool

This stage handles the outer `SB2 ` archive used by the target AGSI game. It inspects,
unpacks, repacks, and verifies the archive structure. It does not extract or modify game
text yet.

## Supported structure

- 44-byte little-endian header (`11 x u32`) with `SB2 ` magic.
- Ordered sections: `CODE`, `TTBL`, `FTBL`, `FTBL`, `VTBL`, `CSTR`, `CDBL`, `DBG_`, `DBG_`.
- Section boundaries derived from header counts and each section's records.
- CSTR boundaries derived from its offset/size table and validated against the file bounds.
- Unknown trailing bytes are preserved as an opaque `TAIL.bin` section.
- A zero-byte `VTBL.bin` is valid when the header's VTBL record count is zero.

The unpack directory contains `header.bin`, one payload file per section, and UTF-8
`manifest.json`. Section payload files omit their four-byte tag. The manifest records
the stable order, offsets, sizes, source hash, and relative file roles, so moving the
dump directory does not break packing.

## Usage

Double-click the executable for a persistent interactive menu. Dragging one archive or
dump directory onto it prefills the path but does not perform a write before confirmation.
After an operation, cancellation, or recoverable error, the program returns to the main
menu.

Fully parameterized commands are one-shot:

```powershell
sinfonia-sb2-tool.exe inspect <GAME_ARCHIVE.sb>
sinfonia-sb2-tool.exe unpack <GAME_ARCHIVE.sb> <OUTPUT_DIR>
sinfonia-sb2-tool.exe pack <DUMP_DIR> <OUTPUT_ARCHIVE.sb> --compare-original <GAME_ARCHIVE.sb>
sinfonia-sb2-tool.exe verify <GAME_ARCHIVE.sb> <DUMP_DIR>
```

Use `--overwrite` only with an explicit non-interactive write. Archive sources are never
overwritten implicitly. A non-empty output directory can be replaced only when it has a
valid manifest created by this tool.

## Verification

The target archive must pass all of these checks before script/text work starts:

1. Every section tag and record stays inside the archive.
2. CSTR table entries and the derived pool stay in range.
3. Unchanged `archive -> unpack -> pack` output is byte-exact.
4. Re-unpacking the rebuilt archive produces the same ordered payloads.

## Current limits

- Only the confirmed AGSI `SB2 ` layout above is supported.
- The loader code confirms non-empty TTBL-member and VTBL record widths, but the analyzed
  archive contains no TTBL members and zero VTBL records. Synthetic tests cover these paths;
  another real archive is still desirable for cross-sample validation.
- Section record semantics not required for boundaries are preserved but remain opaque.
- Packing permits payload size changes only when the existing header/count structure still
  parses successfully. Text injection and string-table rebuilding belong to a later stage.
- The tool currently processes one archive or dump per operation.

