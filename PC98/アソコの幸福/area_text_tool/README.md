# Asoko no Shiawase text tool

`area_text` extracts and injects text from the four PC-98 disks of
`アソコの幸福`.

It handles the supported `AREA*.DAT`, `.MES`, `.EXE`, and `BUNSYO.DAT` files.
Directory mode processes a complete unpacked disk tree; single-file mode is
also available.

## Build

```powershell
cargo build --release --bins
```

## Usage

Extract text to UTF-8 JSON:

```powershell
area_text.exe extract --input "<UNPACKED_ROOT>" --output "<JSON_DIR>"
```

Inject translated JSON into a new copy of the unpacked tree:

```powershell
area_text.exe inject `
  --source "<UNPACKED_ROOT>" `
  --translation "<JSON_DIR>" `
  --output "<INJECTED_ROOT>"
```

Add `--overwrite` to replace an existing output. Running the tool with no
arguments, or with one path, opens the interactive menu.

## Translation JSON

```json
{
  "scr_msg": "Original text",
  "message": "Translated text"
}
```

Edit only `message`. `scr_msg` and fields beginning with `_` are used to find
and validate the source text. This project does not use a `name` field.

The tool removes original layout line breaks from JSON and restores the required
layout during injection. `○` and `△` inside AREA messages may be edited.

Simplified Chinese text is written through the tool's built-in CP932 carrier
mapping and requires the matching redrawn `FREECG98.BMP`.

## Limits

- Only the supported structured text regions are extracted.
- AREA pools are limited by their 16-bit offsets.
- Fixed-layout EXE, INTER, and `BUNSYO.DAT` text must fit their available space.
- Rebuilding FDI images is handled by `fdi_pack`.
