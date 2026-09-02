# FOXY2 PD7 converter

`foxy2-pd7` decodes the proprietary 16-colour PD7 graphics used by the PC-98
version of elf's *FOXY2*. It converts one file or a directory tree to PNG without
modifying the source files.

## Build

```powershell
cargo build --release --bins
```

The release executable is `target/release/foxy2-pd7.exe`.

## Use

```powershell
foxy2-pd7 decode <INPUT.PD7> <OUTPUT.PNG>
foxy2-pd7 decode <INPUT_DIR> <OUTPUT_DIR>
foxy2-pd7 decode <INPUT_DIR> <OUTPUT_DIR> --crop --overwrite
foxy2-pd7 inspect <INPUT>
```

Directory conversion is recursive and preserves relative paths. PNGs use a
faithful 640x400 PC-98 canvas by default. `--crop` trims each PNG to the union of
its encoded blocks. Existing output is rejected unless `--overwrite` is supplied.

With no arguments, or with only an input path, the program opens an interactive
flow. It displays the input, output, crop and overwrite choices and asks for final
confirmation before writing.

## Format notes and limitations

- PD7 stores one or two independently compressed rectangular blocks.
- X and width are measured in 8-pixel byte-columns; Y and height are scanlines.
- Each block carries a 16-entry, 12-bit PC-98 palette and a 16-byte opcode table.
- The decoder reproduces the palette brightness mapping and four planar VRAM
  channels used by the original `PUTD7.EXE` viewer.
- Decoding stops when the block rectangle is full, as the original viewer does.
  Corpus files may retain one alignment byte or a three-byte `00 00 1A` DOS EOF
  tail; the tool reports these bytes but does not treat them as an error.
- Output outside the encoded blocks is palette index zero. The format does not
  provide a confirmed transparency mask, so the PNG is opaque.
- This version decodes PD7 to PNG. PNG-to-PD7 encoding is not implemented.
