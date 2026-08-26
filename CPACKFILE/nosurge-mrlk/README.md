# Ciel nosurge DX MRLK archive tool

`nosurge-mrlk` lists, unpacks, and repacks the `MRLK` resource containers used
by the Windows release of **Ciel nosurge DX**. Some of these files use a
`.psarc` extension, but they are not Sony PSARC files.

The tool validates the complete header, file table, CRLF-delimited name table,
payload ranges, and output paths before writing. G1T and other contained files
are copied byte for byte; texture conversion is intentionally outside this
tool.

## Use

Double-click `nosurge-mrlk.exe`, or drag an MRLK `.psarc` onto it, to use the
interactive menu. A path-only launch only prefills the archive path. Unpacking
or packing does not begin until the paths and overwrite choice are shown and
confirmed.

Fully specified commands do not prompt:

```text
nosurge-mrlk list <ARCHIVE.psarc> --limit 20
nosurge-mrlk unpack <ARCHIVE.psarc> <OUTPUT_DIR>
nosurge-mrlk pack <ORIGINAL_ARCHIVE.psarc> <EDITED_DIR> <NEW_ARCHIVE.psarc>
```

Packing uses the original archive as a template for the exact filename table
and entry order. It reads only files named by that template, so unrelated files
in `<EDITED_DIR>` are ignored. Every required file must exist. File sizes may
change: the tool rebuilds all absolute offsets and sizes and validates the
finished archive before installing it.

The source/template archive is never overwritten, even with `--overwrite`.
Existing output is refused unless `--overwrite` is supplied or replacement is
approved interactively. A complete temporary output is built first; when an
existing output is replaced, it is held as a backup until the new output has
been installed.

## Suggested texture workflow

```text
1. Unpack the MRLK archive.
2. Convert and edit the desired G1T files with a separate G1T tool.
3. Put the rebuilt G1T files back under the same relative names.
4. Pack with the untouched original MRLK archive as the template.
5. Test the newly named output archive before replacing any game resource.
```

## Build

Install a current stable Rust toolchain, then run:

```text
cargo build --release
```

The executable is written to `target\release\nosurge-mrlk.exe`.

## Supported format and limits

- 24-byte little-endian header: `MRLK`, zero reserved field, table end, file
  count, name-table offset, and name-table size.
- Eight-byte entries containing an absolute 32-bit offset and 32-bit size.
- UTF-8/ASCII filenames separated and terminated by CRLF. The game samples use
  ASCII filenames.
- Contiguous, uncompressed, unencrypted payloads with no archive-level
  alignment padding.
- Rebuilt archives are limited to the format's 32-bit offsets and sizes.
- Unsafe, duplicate, truncated, overlapping, gapped, or otherwise inconsistent
  archives are rejected. The one standard `PSAR` sample observed in the game is
  a different format and is not handled by this tool.

