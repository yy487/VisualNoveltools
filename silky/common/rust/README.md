# Silky archive and MES tools

These Rust programs replace the older Python workflow:

- `unpack` extracts a Silky ARC archive.
- `repack` rebuilds an archive from an extracted directory.
- `extract` converts MES text to UTF-8 JSON.
- `inject` writes translated JSON back to MES files.

## Build

```powershell
cargo build --release --offline --bins
```

## Archive usage

```powershell
unpack.exe <SCRIPT.ARC> <UNPACKED_DIR>
repack.exe <UNPACKED_DIR> <OUTPUT.ARC>
```

The archive tools support the profiled `silky-lzss` and `garbro-fixed`
layouts. Keep `.silky_arc_manifest.json` when it is generated.

## MES usage

```powershell
extract.exe <MES_FILE_OR_DIR> <JSON_FILE_OR_DIR>
inject.exe <MES_FILE_OR_DIR> <JSON_FILE_OR_DIR> <OUTPUT_FILE_OR_DIR>
```

Directory mode is non-recursive and accepts `-j` for parallel work.

Edit `message`. If `name` is present, it may also be translated. Keep
`scr_msg`, `_scr_name`, and fields beginning with `_` unchanged. The literal
two-character `\n` token is a script control and its count must stay unchanged.
Ruby readings are replaced with full-width spaces while the base text is kept.

## Limits

- Only the two documented ARC layouts and profiled MES instructions are
  supported.
- MES text is CP932 and cannot contain real CR/LF or NUL.
- Instructions cannot be added, removed, or reordered.
- Archive entries not described by the manifest may not preserve unknown gaps.
