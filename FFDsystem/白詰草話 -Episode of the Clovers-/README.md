# Shirotsume Souwa SE / RepiPack v2

Standalone game adapter for the v2 `RepiPack` archives and extracted `mes/*.txt` scripts.
The v2 parser follows the exported executable: archive basename guard, 0x50-byte index entries,
rolling DWORD decoding, the two entry data transforms, and the shared LZSS decoder.

## Archive unpack

The v2 index stores resource paths directly after decoding. It does not use the v5 basename-MD5
index. Output preserves relative paths under `files/`; unsafe names and failed entries are kept as
packed bytes under `unresolved/`. `manifest.tsv` records offsets, sizes, transform type, and paths.

```powershell
cargo build --release --offline

.\target\release\repi_unpack_v2.exe unpack `
  --archive '<game-dir>\B.dat' `
  --output '<game-dir>\B_unpacked'
```

## Script text export and injection

`export` scans `.txt` scripts for text in `CreateBalloon`, `CreateBalloonEx`, `CreateBalloonBie`,
`CreateText`, and `AddText`. Balloon text is the ninth argument, `CreateText` text is the seventh,
and `AddText` text is the second. `CreateBalloonBie` supplies choice labels used by the surrounding
`select`/`case` branches. The translation JSON is one UTF-8 array per source file and contains only
`message` fields. `source-map.tsv` binds each JSON path to its source file and content fingerprint.

The visible message removes `<SOUND>`, `<TYPE>`, and `<FONT>` formatting tags; `<BR>` becomes a
newline. On import, the adapter validates the source fingerprint and JSON shape, injects by the
original string-byte spans, converts newlines back to `<BR>`, and restores the source formatting
tags. When translated text changes length, non-visible tags are relocated proportionally within
the edited message; unchanged messages retain their original bytes. All output is a separate full
script directory; source files and translation JSON stay untouched.

```powershell
.\target\release\repi_unpack_v2.exe export `
  --source-dir '<game-dir>\mes\files' `
  --output-dir '<game-dir>\mes\translations'

.\target\release\repi_unpack_v2.exe import `
  --source-dir '<game-dir>\mes\files' `
  --translation-dir '<game-dir>\mes\translations' `
  --output-dir '<game-dir>\mes\rebuilt'
```

