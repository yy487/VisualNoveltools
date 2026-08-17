# Kankin font tools

`kankin_font_patch.py` prepares CP932 carrier text and redraws the matching
glyphs in this game's `font.dat`.

## Requirements

```powershell
pip install pillow
```

## Usage

Check the font and carrier map:

```powershell
python kankin_font_patch.py info <FONT.DAT>
python kankin_font_patch.py scan <JSON_DIR> --cn-jp <MAP.JSON> --output scan_report.json
```

Convert translated JSON to CP932 carrier characters:

```powershell
python kankin_font_patch.py convert-json <JSON_DIR> --cn-jp <MAP.JSON> --output-dir <CARRIER_JSON_DIR>
```

Redraw the carrier glyphs:

```powershell
python kankin_font_patch.py patch <FONT.DAT> <OUTPUT.DAT> --json <JSON_DIR> --cn-jp <MAP.JSON> --font <FONT_FILE>
```

Use the original Chinese JSON for `patch` and the converted carrier JSON for
script injection. By default, only `name` and `message` are converted. Do not
convert `scr_msg` or source-location fields.

`--scale`, `--padding`, `--font-size`, `--x-offset`, and `--y-offset` adjust the
redrawn glyphs. `--slots` limits the font sections to patch.

## Limits

- Every carrier character must be CP932-encodable and present in `font.dat`.
- One carrier glyph cannot represent several Chinese characters.
- This tool does not inject scripts or patch the executable's text encoding.
