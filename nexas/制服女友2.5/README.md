# Uniform Kanojo NeXAS text tools

These Rust tools handle the unpacked NeXAS `mes` scripts used by
`Uniform Kanojo 2.5`.

- `nexas_extract` extracts text to UTF-8 JSON.
- `nexas_inject` writes translations back and rebuilds script tables.
- `nexas_rebuild` checks an unchanged parse and rebuild.

## Build

```powershell
cargo build --release --offline --bins
```

## Usage

```powershell
nexas_extract.exe "<MES_DIR>" --output "<OUTPUT_DIR>\mes_json"
nexas_inject.exe "<MES_DIR>" "<OUTPUT_DIR>\mes_json" `
  --output "<OUTPUT_DIR>\mes_injected"
nexas_rebuild.exe "<MES_DIR>" --output "<OUTPUT_DIR>\mes_rebuilt"
```

The commands also accept one `.bin` file. Directory injection copies the full
source tree and replaces only scripts with matching JSON. Existing output is
rejected.

## Translation JSON

Edit `message`. If `name` is present, it may also be translated. Keep
`scr_msg`, `_scr_name`, `_scr_raw`, control fields, and all other fields
beginning with `_` unchanged.

Controls inside `message`, including `@n`, may be added, removed, or reordered
as long as their syntax is valid. Prefix and suffix controls outside the body
are restored from the source. Messages and names may grow or shrink.

## Limits

Only unpacked UTF-8 `VER-1.00` scripts are supported. The tool does not rebuild
the outer PAC archive. `__global.bin` and invalid UTF-8 strings are preserved
rather than translated. Unprofiled system-text rules and control meanings
require review before using the tool with another game.
