# Asoko no Shiawase FDI unpacker

`fdi_unpack` extracts one or more PC-98 FDI/FAT12 disk images into a managed
workspace.

## Build

```powershell
cargo build --release --bin fdi_unpack
```

## Usage

```powershell
fdi_unpack.exe unpack `
  --input "<GAME_DIR>\disk_1.fdi" `
  --input "<GAME_DIR>\disk_2.fdi" `
  --output "<OUTPUT_DIR>"
```

Each disk is extracted to its own subdirectory. `workspace.json` records the
disk and file layout needed by `fdi_pack`.

Add `--overwrite` to replace an existing workspace created by this tool.
Running the tool with no arguments, or passing FDI paths without `unpack`, opens
the interactive menu.

## Limits

- Supports uncompressed FDI images containing a consistent FAT12 filesystem.
- Uses CP932 DOS 8.3 names; unsupported or unsafe Windows paths are rejected.
- Does not restore DOS timestamps on extracted host files.
- Does not handle FAT long file names.
