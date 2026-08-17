# Kokuonshoku script tools

`extract.py` and `inject.py` handle the CP932 `.s` scripts used by `刻音色`.
They extract messages and choices to UTF-8 JSON and rebuild translated scripts.

## Usage

```powershell
python extract.py <SCRIPT_DIR> <JSON_DIR>
python inject.py <SCRIPT_DIR> <JSON_DIR> <OUTPUT_DIR> --stats-json <REPORT.JSON>
```

The same commands accept one script and one JSON file. Use `--name-map` during
extraction to add speaker context from voice prefixes.

## Translation JSON

Edit only `message`. Keep `scr_msg` and fields beginning with `_` unchanged.
The game has no writable speaker-name slot.

Internal `＃` page marks are hidden from `message`. The current injector
defaults to `--page-mark-mode drop --layout-policy warn`: it removes internal
page marks and reports text that may exceed the renderer. Use `auto-fit` to add
safe breaks automatically, `manual` to keep marks written in `message`, or
`--layout-policy skip` to omit unsafe entries.

The default `relocate` mode rebuilds messages and known jump offsets. Use
`--mode in-place` only when output must keep the original file size.

## Limits

- Message and choice payloads are limited to 255 encoded bytes.
- Text must be CP932. The renderer expects two-byte characters; half-width text
  can be unsafe.
- Only the known absolute-offset opcodes are relocated.
- Outer archive handling and font work are not included.
