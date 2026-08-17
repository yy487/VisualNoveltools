# Asoko no Shiawase FDI packer

`fdi_pack` rebuilds one or more PC-98 FDI/FAT12 images from a workspace created
by `fdi_unpack`.

## Build

```powershell
cargo build --release --bin fdi_pack
```

## Usage

Pack a single disk or a complete unpacked workspace:

```powershell
fdi_pack.exe pack `
  --source "<ORIGINAL_FDI_OR_DIR>" `
  --unpacked "<UNPACKED_ROOT>" `
  --output "<OUTPUT_FDI_OR_DIR>"
```

The source and output must both be files for a single disk, or both be directory
roots for a batch. Add `--overwrite` to replace existing output files.

The older explicit single-disk form is also supported:

```powershell
fdi_pack.exe pack `
  --source "<ORIGINAL_FDI>" `
  --workspace "<WORKSPACE_JSON>" `
  --files "<MEMBER_DIR>" `
  --output "<OUTPUT_FDI>"
```

Running the tool with no arguments, or with path arguments only, opens the
interactive menu.

## Limits

- The original FDI must match the workspace used for unpacking.
- Existing files may grow, shrink, or become empty when disk space permits.
- Files and directories cannot be added, removed, or renamed.
- Supports uncompressed FDI images with the FAT12 layout used by this project.
- Member formats are handled by separate text and resource tools.
