# NECRONOMICON GPC to PNG

`gpc2png` decodes the observed PC-98 `PC98)GPCFILE` format into indexed PNG.
The parser is structure-aware: it reads the GPC header, palette table, image
info block, compressed payload, row restoration metadata, and four bitplanes.
It does not scan arbitrary binary bytes for image-looking data.

## Usage

```powershell
gpc2png.exe E:\GAL\NECRONOMICON_DUMP\DISK_A\GPC `
  --output E:\GAL\NECRONOMICON_DUMP\work\03_gpc2png\png
```

Directory input is recursive. Existing output files are rejected instead of
overwritten. The output is an 8-bit indexed PNG with the 16-color PC-98
palette; no external image library is required.

## Decoding policy

- Magic: `PC98)GPCFILE`.
- Palette: 16 entries, 2-byte PC-98 BGR nibbles expanded to 8-bit RGB.
- Image payload: four bitplanes with the format's interleaving and row
  prediction restoration.
- Compression: the observed control-byte/literal/zero-skip stream, with
  bounds checks and deterministic truncation at the output boundary.
- Unsupported geometry, plane counts, malformed offsets, or truncated data
  produce a warning; the process fails if no image can be converted.

## Verification

The release build passes `cargo fmt -- --check`, `cargo test --offline`,
`cargo clippy --offline --all-targets -- -D warnings`, and
`cargo build --release --offline --bins`. The full extracted GPC corpus
contains 388 files; all 388 converted with zero warnings. Every generated PNG
was checked for signature, IHDR, PLTE, IDAT, IEND, CRC, and valid dimensions.

Source: `src\bin\gpc2png.rs`.
