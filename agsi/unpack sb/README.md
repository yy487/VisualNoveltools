# AGSI SB2 archive tool

`sinfonia-sb2-tool` inspects, unpacks, verifies, and rebuilds the outer `SB2 `
archive used by the target AGSI game.

## Usage

```powershell
sinfonia-sb2-tool.exe inspect <ARCHIVE.SB>
sinfonia-sb2-tool.exe unpack <ARCHIVE.SB> <OUTPUT_DIR>
sinfonia-sb2-tool.exe verify <ARCHIVE.SB> <UNPACKED_DIR>
sinfonia-sb2-tool.exe pack <UNPACKED_DIR> <OUTPUT.SB> --compare-original <ARCHIVE.SB>
```

The unpacked directory contains `manifest.json`, `header.bin`, and one file per
archive section. Keep them together for verification and packing. Running the
program without complete command arguments opens the interactive menu.

## Limits

- Only the profiled `SB2 ` section layout is supported.
- Payload sizes may change only when the existing header and record counts
  still describe a valid archive.
- Section contents are preserved but mostly opaque.
- This tool does not extract text or rebuild script string tables.
