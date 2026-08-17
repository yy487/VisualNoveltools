# Martopia DAT and font tools

These Python tools handle the DAT archives and `SjisFont` bitmap font used by
side-B's `Martopia`.

- `dat_tool.py` unpacks and rebuilds DAT archives.
- `martopia_cnjp_font_tool.py` converts translated JSON to CP932 carrier text
  and redraws the matching font glyphs.

## Requirements

```powershell
pip install pillow numpy
```

## DAT usage

```powershell
python dat_tool.py unpack <INPUT.DAT> <UNPACKED_DIR> --meta <META_DIR>
python dat_tool.py pack <UNPACKED_DIR> <META.JSON> <OUTPUT.DAT>
python dat_tool.py unpack-all <DAT_DIR> <UNPACKED_ROOT> --meta <META_DIR>
python dat_tool.py pack-all <UNPACKED_ROOT> <META_DIR> <OUTPUT_DIR>
python dat_tool.py selftest <INPUT.DAT>
```

Keep the generated metadata with each unpacked archive; it is required for
packing.

## Carrier text and font

```powershell
python martopia_cnjp_font_tool.py check-map <GAME.EXE> <MAP.JSON>
python martopia_cnjp_font_tool.py patch-font <GAME.EXE> <FONT_DIR> <MAP.JSON> <OUTPUT_DIR> --ttf <FONT_FILE>
python martopia_cnjp_font_tool.py convert-json <JSON_DIR> <CARRIER_JSON_DIR> <MAP.JSON> --strict
```

By default, JSON conversion edits `message` and `message_parts`. Add `name` to
`--fields` when speaker names are writable. Keep `scr_msg` unchanged. Use the
converted JSON for script injection and place all four rebuilt `SjisFont` PNG
files back into the unpacked DAT before packing.

## Limits

- The DAT format and font index are specific to this game.
- Every carrier character must be CP932-encodable, present in the font index,
  and assigned to only one Chinese character.
- This tool does not analyze or inject the game's script bytecode.
- Repacked DAT bytes and size may differ because compression and IVs are
  regenerated.
