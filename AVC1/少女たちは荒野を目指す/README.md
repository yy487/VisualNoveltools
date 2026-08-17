# Shoujo-tachi wa Kouya wo Mezasu MES text tools

These Rust tools extract and inject the CP932 `.txt` scripts unpacked from the
game's ACV archive.

- `mes_extract` writes one UTF-8 JSON file per script.
- `mes_inject` creates translated scripts or a translated script directory.

## Build

```powershell
cargo build --release --bins
```

## Usage

```powershell
mes_extract.exe "<MES_DIR>" --output "<OUTPUT_DIR>\mes_json"
mes_inject.exe "<MES_DIR>" "<OUTPUT_DIR>\mes_json" `
  --output "<OUTPUT_DIR>\mes_injected"
```

Both commands also accept one `.txt` file. Directory injection copies the full
source tree and replaces only scripts with matching JSON.

## Translation JSON

Edit `message`. If `name` is present, it may also be translated. Keep
`scr_msg`, `_scr_name`, and fields beginning with `_` unchanged.

Names cannot contain `【`, `】`, `@`, or an ASCII comma. Choice messages cannot
contain an ASCII quote or `,*`; their jump targets are kept from the source.

Text must be encodable as CP932 and cannot contain NUL or real CR/LF. `[n]` is
the display-line control. Existing `[n]` controls may be removed, but new ones
cannot be added and retained controls must stay in source order. Messages,
names, and choices may grow or shrink.

## Limits

Only the profiled CP932, LF `.txt` scripts are supported. This tool does not
unpack or rebuild the outer ACV archive. Control syntax other than `[n]` is not
interpreted.
