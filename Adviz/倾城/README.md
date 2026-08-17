# Keisei ADV localization tools

These Rust tools handle the Adviz for Windows 95 scripts used by `Keisei`.

- `qc_extract` extracts `.ADV` text to UTF-8 JSON.
- `qc_inject` writes translated text back and rebuilds script addresses.
- `qc_anm_info` checks BIZ2/LZSS `.ANM` files and prints their image details.

## Build

```powershell
cargo build --release --bins
```

## Usage

```powershell
qc_extract.exe "<TEXT_DIR>" --output "<OUTPUT_DIR>\text_json"
qc_inject.exe "<TEXT_DIR>" "<OUTPUT_DIR>\text_json" `
  --output "<OUTPUT_DIR>\text_injected"
qc_anm_info.exe "<GAME_DIR>\image.ANM"
```

Extraction and injection also accept one `.ADV` file. Directory injection
copies the source directory and replaces only scripts with matching JSON.
Existing output is rejected.

## Translation JSON

Edit `message`. A static `name` may also be edited when
`_name_writable` is `true`. Keep `scr_msg`, `_scr_name`, and all fields beginning
with `_` unchanged.

The scripts use CP932. Text and choices may grow or shrink, but the rebuilt ADV
must remain below the engine's 65,536-byte file limit.

Source `\n` controls are not shown in JSON. Edited messages rely on the game's
automatic wrapping. Structural suffixes such as `\k\*` and choice prefixes are
restored automatically. Controls left inside `message`, such as `\%...;`, must
keep valid syntax and the order required by the source entry.

## Limits

Only the supported Keisei ADV layout is accepted. Dynamic speaker names are
read-only. Text outside CP932 requires a separate renderer, encoding, or font
patch. `qc_anm_info` only inspects ANM files; it does not convert or rebuild
them.
