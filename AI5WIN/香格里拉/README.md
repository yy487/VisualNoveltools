# Shangri-La 1 MES tools

These Rust tools unpack `MES.ARC`, decompress its scripts, extract text to
UTF-8 JSON, inject translations, and rebuild the archive.

## Build

```powershell
cargo build --release --offline --bins
```

## Workflow

```powershell
unpack.exe --output <UNPACKED_DIR> <MES.ARC>
decompress.exe --output <SCRIPT_DIR> <UNPACKED_DIR>
extract.exe --output <JSON_DIR> <SCRIPT_DIR>
inject.exe --output <INJECTED_DIR> <SCRIPT_DIR> <JSON_DIR>
compress.exe --output <COMPRESSED_DIR> <INJECTED_DIR>
pack.exe --output <OUTPUT.ARC> <MES.ARC> <COMPRESSED_DIR>
```

Edit only `message`. `name` is context taken from the script and is read-only.
Keep `scr_msg` and all fields beginning with `_` unchanged. Messages may grow
or shrink, but the rebuilt decompressed script must stay below 64,000 bytes.

## Limits

- Only this game's flat `MES.ARC`, LZSS stream, and profiled MES bytecode are
  supported.
- Text is CP932 and cannot contain NUL or real CR/LF.
- The long battle-message records are kept as single entries.
- The tools do not patch fonts, encoding, or the executable.
