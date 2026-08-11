# Refrain Blue AI5WIN font fix

This directory contains a cleaned, reproducible toolset for the Refrain Blue
AI5WIN appended-font patch.

## Layout

- `source/`
  - `AI5WIN_chs.exe`: base patched EXE with appended TBL/FNT region.
  - `FONT.FNT`, `FONT.TBL`, `FONT.PAL`: complete appended font source.
  - `char_list.txt`: original character order.
  - `subs_cn_jp.json`: CN -> JP codepoint mapping.
- `tools/`
  - `rb_ai5win_font_fix.py`: project-specific repair and injection helper.
  - `png_to_FNT.py`, `show_FNT.py`, `TBL_exchange.py`, `generate_char_list.py`: original SExtractor AI5WIN utilities.
- `build/`
  - generated FNT outputs.
- `output/`
  - generated EXE outputs.
- `preview/`
  - old-vs-fixed comparison images.

## Current recommended build

The current recommended build redraws the full 7069-glyph appended FNT with
SimHei strong style. The original `FONT.FNT` is used only as the slot/layout
source. Rare symbols missing from SimHei are redrawn with the configured
fallback font chain, currently `msgothic.ttc` first.

Generated output:

```text
output/AI5WIN_chs_simhei_strong_full_plus_dong_rare.exe
build/FONT_simhei_strong_full_plus_dong_rare.FNT
preview/simhei_strong_full_redraw_sentence_preview.png
preview/simhei_strong_full_plus_dong_rare_cluster.png
preview/simhei_strong_full_plus_dong_rare_focus.png
```

`simhei_strong_full_redraw_sentence_preview.png` is the main visual check: it
compares original old FNT rendering against the full SimHei redraw on complete
sample lines. The focused preview is only a slot-level diagnostic for the
previously problematic cluster:

```text
472 神
473 》
474 配
475 曇 -> 昙
476 途
477 額 -> 额
478 汗
479 拭
7065 鵫 -> 鸫
```

## Extra glyph slot

`鸫` is not encodable in CP932 and is not present in `FONT.TBL`. The project
uses a rare unused CP932 slot as a surrogate:

```text
鸫 -> 鵫
鵫 index: 7065
鵫 CP932: EEE9
```

This override is stored in:

```text
source/extra_cn_jp.json
```

The full SimHei strong output includes this extra `鸫` slot:

```text
output/AI5WIN_chs_simhei_strong_full_plus_dong_rare.exe
build/FONT_simhei_strong_full_plus_dong_rare.FNT
preview/simhei_strong_full_plus_dong_rare_focus.png
```

## Rebuild

Run from `tools/` with a relative script path. Avoid launching Python with an
absolute path containing Japanese directory names in non-Unicode consoles.

```powershell
Set-Location -LiteralPath '<FONT_FIX_TOOLS_DIR>'
python .\rb_ai5win_font_fix.py --full-redraw --style strong --out-fnt ..\build\FONT_simhei_strong_full_plus_dong_rare.FNT --out-exe ..\output\AI5WIN_chs_simhei_strong_full_plus_dong_rare.exe --preview ..\preview\simhei_strong_full_plus_dong_rare_cluster.png
```

Optional parameters:

```powershell
python .\rb_ai5win_font_fix.py --indices 472-479 --font "$env:WINDIR\Fonts\simhei.ttf" --font-size 22 --outline 2
python .\rb_ai5win_font_fix.py --indices 472-479,7065 --style strong --out-fnt ..\build\FONT_simhei_strong_plus_dong_rare.FNT --out-exe ..\output\AI5WIN_chs_simhei_strong_plus_dong_rare.exe --preview ..\preview\simhei_strong_plus_dong_rare.png
python .\rb_ai5win_font_fix.py --full-redraw --style strong --fallback-fonts "$env:WINDIR\Fonts\msgothic.ttc,$env:WINDIR\Fonts\meiryo.ttc"
```

## EXE layout

The helper preserves the existing CHS EXE layout:

```text
TBL file offset: 0x18C600
FNT file offset: 0x18FE00
FNT reserved size: 0x2C0400
TBL VA: 0x00E69800
FNT VA: 0x00E6D000
```

The script verifies that the source `FONT.TBL` matches the EXE's appended TBL
region before writing the new FNT region.

## Dependencies

- Python 3
- Pillow
- fontTools

The original SExtractor utilities may additionally require `numpy` and `tqdm`.
