# raiL-soft XFL and GSC localization tools

These Rust tools unpack raiL-soft XFL archives, extract GSC text to UTF-8 JSON,
inject translations, and rebuild the archive.

## Build

```powershell
cargo build --release --offline
```

## Workflow

```powershell
xfl_unpack.exe <SCRIPT.XFL> --no-pause
gsc_extract.exe <UNPACKED_DIR> --no-pause
gsc_inject.exe <JSON_DIR> --no-pause
xfl_pack.exe <REBUILT_DIR> --no-pause
```

The programs choose adjacent output paths. Keep `.xfl-manifest.json` and
`.gsc-manifest.json`; they are required for packing and injection.

For explicit paths and archive listing, use:

```powershell
railsoft-xfl.exe list <ARCHIVE.XFL>
railsoft-xfl.exe unpack <ARCHIVE.XFL> <OUTPUT_DIR>
railsoft-xfl.exe pack <INPUT_DIR> <OUTPUT.XFL>
```

## Translation JSON

Edit `message`. If `name` is present, it may also be translated. Keep
`scr_msg` and fields beginning with `_` unchanged. Names cannot be added to
entries that did not have one.

Script `^n` controls are shown as normal JSON line breaks and restored during
injection. Ruby readings are removed, leaving only the base text. Re-extract
old translation JSON before using this version if it still contains `^n`,
`|base[reading]`, or inline ruby brackets.

## Limits

- Only the supported raiL-soft XFL v1 and GSC layouts are accepted.
- Text is written as CP932; Chinese requires a separate carrier map and font.
- The tools rebuild the text pool but do not disassemble or rewrite the full VM.
