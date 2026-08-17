# Silky localization tools

These Python tools unpack Silky ARC archives, extract and inject MES text, and
build a CP932 carrier font for `肢体を洗う`.

## Requirements

Python 3 is required. Pillow and fontTools are needed for the BFD font tools.

## Workflow

```powershell
python silky_arc_tool.py unpack <SCRIPT.ARC> <SCRIPT_DIR>
python silky_mes_extract.py extract <SCRIPT_DIR> <JSON_DIR>
python silky_bfd_font.py scan-json <JSON_DIR> <CHARSET.JSON>
python silky_bfd_font.py make-map <CHARSET.JSON> <FONT.BFD> <MAP.JSON> --subs <SUBS.JSON>
python silky_bfd_font.py build <FONT.BFD> <MAP.JSON> <FONT_FILE> <OUTPUT.BFD> --preview <PREVIEW.PNG>
python silky_mes_inject.py <SCRIPT_DIR> <JSON_DIR> <PATCHED_DIR> --map <MAP.JSON> --copy-unmodified
python patch_silky_bfd_exe_read_limit.py <GAME.EXE> <OUTPUT.EXE> --font <OUTPUT.BFD>
python silky_arc_tool.py pack <PATCHED_DIR> <OUTPUT.ARC>
```

Edit `message`. If `name` is present, it may also be translated. Keep
`scr_msg`, `_scr_name`, and fields beginning with `_` unchanged.

The carrier map writes CP932-compatible characters to scripts and redraws
their BFD glyphs as the requested Chinese characters. The same map must be used
for font building and MES injection.

## Limits

- The tools support the profiled Silky ARC, MES, and BFD layouts.
- Text outside CP932 requires a unique carrier mapping and matching glyph.
- A larger BFD may require the supplied executable read-limit patch.
- `--scan-all` can include non-dialogue strings and should be reviewed before
  injection.
