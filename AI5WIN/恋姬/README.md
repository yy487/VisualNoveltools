# Koihime AI5WIN MES tools

`extract.py` and `inject.py` extract and rebuild the profiled AI5WIN `.MES`
scripts.

## Usage

```powershell
python extract.py <MES_FILE_OR_DIR> <JSON_FILE_OR_DIR>
python inject.py <MES_FILE_OR_DIR> <JSON_FILE_OR_DIR> <OUTPUT_FILE_OR_DIR> --copy-unmatched
```

Edit `message`. Static `name` values may also be translated; use
`--no-inject-names` to keep the original names. Keep `scr_msg` and fields
beginning with `_` unchanged.

The runtime protagonist call is exposed as the literal name `小十郎`. For
multipart entries, edit the top-level `message`; `message_parts` is retained
for compatibility and structure context. Ruby readings are blanked while the
base text is kept. Supported visible gaiji may be written as their symbols or
as tokens such as `{{EB:A4}}`.

## Limits

- Only profiled opcode-1 dialogue, narration, and choices are exported.
- System text and function arguments are excluded.
- Text must use the supported script encoding and gaiji mapping.
- Only known absolute jump targets are relocated during rebuilding.
