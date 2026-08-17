# NECRONOMICON FDI repacker

`fdi_repack` replaces existing files in a PC-98 FAT12 disk image and writes a
new FDI.

Use `mes_inject` first, then pass the matching disk subtree as the replacement
root.

## Build

```powershell
cargo build --release --bin fdi_repack
```

## Usage

```powershell
fdi_repack.exe "<ORIGINAL.FDI>" `
  --replacements "<REPLACEMENT_ROOT>" `
  --output "<OUTPUT.FDI>"
```

Replacement paths are relative to the disk root. For example, a replacement
for `MES/OPEN1.MES` must appear at `<REPLACEMENT_ROOT>/MES/OPEN1.MES`.

## Limits

- Replacement files must already exist in the image and use valid CP932 8.3
  paths.
- Files may grow, shrink, or become empty when disk space permits.
- New files, renamed files, long file names, and ambiguous paths are rejected.
- The input must use the FAT12 FDI layout supported by this project.
- The output path must not already exist.
- This tool does not parse MES text or modify fonts.
