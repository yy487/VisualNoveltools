# NECRONOMICON GPC to PNG

`gpc2png` converts the game's PC-98 `.GPC` images to indexed PNG files.

## Build

```powershell
cargo build --release --bin gpc2png
```

## Usage

Convert one file or recursively convert a directory:

```powershell
gpc2png.exe "<GPC_FILE_OR_DIR>" --output "<PNG_FILE_OR_DIR>"
```

Directory conversion keeps the input directory structure. Existing output
files are rejected.

## Limits

- Supports the observed `PC98)GPCFILE` format with a 16-color palette and four
  bitplanes.
- Malformed or unsupported images are reported and skipped.
- This tool only decodes GPC to PNG; it does not encode PNG back to GPC.
