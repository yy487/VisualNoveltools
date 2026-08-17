# NECRONOMICON MES text tools

These tools extract and inject text from the PC-98 version of `NECRONOMICON`.

- `mes_extract` converts `.MES` scripts to UTF-8 JSON.
- `mes_inject` rebuilds `.MES` scripts from translated JSON.

## Build

```powershell
cargo build --release --bins
```

## Usage

Extract one file or recursively extract a directory:

```powershell
mes_extract.exe "<MES_FILE_OR_ROOT>" --output "<JSON_FILE_OR_DIR>"
```

Inject a JSON file or directory:

```powershell
mes_inject.exe "<JSON_FILE_OR_DIR>" `
  --source-root "<MES_ROOT>" `
  --output "<OUTPUT_FILE_OR_DIR>"
```

The source root must contain each source file at the relative path stored in its
JSON `_file` field. Existing outputs are rejected.

## Translation JSON

```json
{
  "_scr_name": "Original name",
  "name": "Translated name",
  "scr_msg": "Original message",
  "message": "Translated message"
}
```

Edit `message`, and edit `name` only when the entry already contains it. Do not
change `scr_msg`, `_scr_name`, `_file`, or other fields beginning with `_`.
Entries without a name cannot be given one.

Text is encoded as CP932 with the game's custom hiragana bytes. Messages may
grow or shrink, but NUL, CR/LF, invalid encoding, and bytes that conflict with
MES opcodes are rejected.

## Limits

- Use a clean A-K disk tree extracted from the original images; do not use the
  older mixed-disk dump tree.
- The `D5` runtime stream transformation is preserved but not simulated.
- The tool does not modify fonts or rebuild FDI images.
