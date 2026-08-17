# Majikoi S script tools

These Python tools handle NeXAS `.bin` scripts from
`真剣で私に恋しなさい！S`.

- `nexas_disasm.py` writes a readable script dump.
- `nexas_extract.py` extracts dialogue and choices to UTF-8 JSON.
- `nexas_inject.py` rebuilds scripts from translated JSON.

## Usage

```powershell
python nexas_disasm.py <BIN_DIR> -o <DISASM_DIR>
python nexas_extract.py <BIN_DIR> -o <JSON_DIR>
python nexas_inject.py <BIN_DIR> --json <JSON_DIR> -o <OUTPUT_DIR> --encoding cp932
```

Use `--encoding gbk` only when the target game has been patched to read GBK.

## Translation JSON

Edit `message`. `name` may also be edited. Keep fields beginning with `_`
unchanged; they identify string slots, choices, control tags, and split
dialogue parts during rebuilding.

## Limits

- The JSON does not contain a separate immutable source-text field.
- Extraction depends on the dialogue and choice patterns implemented for this
  game. Unrecognized string uses are not exported.
- Injection accepts CP932 or GBK only and operates on a directory of `.bin`
  files.
