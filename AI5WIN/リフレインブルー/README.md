# Refrain Blue text and font tools

These Python tools unpack `MES.ARC`, extract its MES text to JSON, inject
translations, and rebuild the archive. Separate helpers dump and rebuild the
game's FNT/PAL/TBL bitmap font.

## Requirements

```powershell
pip install -r requirements.txt
```

## Text workflow

```powershell
python rp_workflow.py unpack-extract <MES.ARC> <WORK_DIR>
python rp_workflow.py inject-pack <MES.ARC> <WORK_DIR> <OUTPUT.ARC>
```

The first command creates `mes_raw` and `json` inside the work directory.
Edit only `message` in each JSON entry. Keep `scr_msg` and fields beginning
with `_` unchanged. This profile has no writable `name` field.

The longer form is also available:

```powershell
python rp_workflow.py unpack <MES.ARC> <MES_DIR>
python rp_workflow.py extract <MES_DIR> <JSON_DIR>
python rp_workflow.py inject <MES_DIR> <JSON_DIR> <INJECTED_DIR>
python rp_workflow.py pack <INJECTED_DIR> <OUTPUT.ARC> --base-arc <MES.ARC>
```

Text must be encodable as CP932. Longer messages use an end-of-file jump
block; records too short to hold the jump cannot grow.

## Font tools

```powershell
python rp_font_dump.py <FONT.FNT> <FONT.PAL> <FONT.TBL> <GRID.PNG>
python rp_font_build.py <GRID.PNG> <CHAR_LIST.TXT> <NEW.FNT> <NEW.PAL> <NEW.TBL>
python rp_font_make_charset.py <JSON_DIR> <CHARSET.TXT>
```

`rp_font_patch_exe.py` is experimental and should only be used on a separate
copy of the executable.

## Limits

- The MES scanner handles the profiled opcode-1 CP932 strings, not the full VM.
- Ruby text is skipped unless `--include-ruby` is used.
- Choice strings are not classified separately.
- Characters outside CP932 require a carrier map and matching font changes.
