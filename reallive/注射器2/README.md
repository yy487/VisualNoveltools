# RealLive Seen.txt tools

These Python tools extract, inject, and verify text in the project's RealLive
`Seen.txt`.

## Usage

```powershell
python extract.py <SEEN.TXT> <JSON_DIR> --clean
python inject.py <SEEN.TXT> <JSON_DIR> <OUTPUT.TXT>
python verify.py <OUTPUT.TXT> --original <SEEN.TXT> --json <JSON_DIR>
```

Use `--seen 2 106` to process selected SEEN blocks. If injection uses a carrier
map, pass `--map-json <MAP.JSON>` to both injection and verification.

## Translation JSON

Edit `message`. If `name` and `_scr_name` are present, `name` may also be
translated. Keep `scr_msg`, `_scr_name`, and fields beginning with `_`
unchanged.

The injector rebuilds changed VM code and updates the supported inline flow
targets.

## Limits

- Only the profiled RealLive compression, encryption key, text records, and
  jump-table forms are supported.
- Text encoding and carrier mappings must match the target executable and font.
- Use `verify.py` after injection; unsupported control-flow forms require
  further analysis rather than binary text replacement.
