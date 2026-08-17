# Refrain Blue font fix

`rb_ai5win_font_fix.py` redraws the appended AI5WIN FNT glyphs and writes them
into the prepared Chinese executable.

## Requirements

```powershell
pip install pillow fonttools
```

## Usage

Run from the `tools` directory:

```powershell
python .\rb_ai5win_font_fix.py --full-redraw --style strong `
  --out-fnt ..\build\FONT_redrawn.FNT `
  --out-exe ..\output\AI5WIN_redrawn.exe `
  --preview ..\preview\font_preview.png
```

Use `--font`, `--font-size`, and `--outline` to select the primary font style.
Use `--fallback-fonts` for missing glyphs or `--indices` to redraw selected
slots only.

The project-specific extra carrier mapping for `鸫` is stored in
`source/extra_cn_jp.json` and uses the existing `鵫` slot.

## Limits

- The script expects the prepared Refrain Blue executable and appended
  TBL/FNT layout supplied in `source`.
- It validates the embedded TBL before writing the new font region.
- Other AI5WIN executables or font layouts are not supported.
