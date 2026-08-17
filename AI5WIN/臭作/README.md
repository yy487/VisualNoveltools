# Shuusaku ARC and GPX tools

These Python tools handle the old AIWIN/ELF resources used by `臭作`.

- `aiwin_arc_extract.py` lists and extracts ARC archives.
- `gpx2png.py` converts extracted GPX images to PNG.

## Requirements

```powershell
pip install pillow
```

## ARC usage

```powershell
python aiwin_arc_extract.py list <ARCHIVE.ARC>
python aiwin_arc_extract.py extract <ARCHIVE.ARC> <OUTPUT_DIR>
```

The default `auto` mode decompresses `.MES` entries and writes other entries,
including `.GPX`, as stored. Use `--mode raw` or `--mode decompress` to
override this choice.

## GPX usage

```powershell
python gpx2png.py <IMAGE.GPX> <IMAGE.PNG>
python gpx2png.py <GPX_DIR> <PNG_DIR> --overwrite
python gpx2png.py <GPX_DIR> <PNG_DIR> --transparent-index 2
```

Use `--auto-transparent` for conservative transparency detection. Use
`--palette-order bgr` only for a variant whose red and blue channels are
reversed.

## Limits

- ARC rebuilding is not supported.
- Automatic ARC decompression is selected by file extension; unknown
  compressed member types need an explicit mode.
- GPX conversion is one-way. PNG files cannot be written back to GPX.
- Transparency is not enabled by default.
