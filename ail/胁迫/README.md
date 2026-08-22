# AIL instruction-based localization tool

A Rust command-line tool for the AIL script format used by this project. It migrates the existing translation by exact bytecode reference, exports UTF-8 JSON, rebuilds variable-length CP932 text safely, and repacks the outer SNL archive.

The tool always ignores `0047.bin` for text work. During injection it copies that file unchanged.

## Build

```powershell
cargo build --release
```

The executable is `target\release\ail_text_tool.exe`.

## Recommended workflow

### 1. Recover the existing translation

```powershell
ail_text_tool.exe migrate `
  --source <ORIGINAL_BIN_DIR> `
  --legacy <LEGACY_TRANSLATED_BIN_DIR> `
  --output <TRANSLATION_JSON_DIR>
```

This compares the matching original and legacy BIN files, recovers changed string operands, validates every operand against the confirmed opcode grammar, and writes one UTF-8 JSON file per BIN.

Matching is by `_file + _ref_offset`, not by source text. This preserves occurrence-specific translations.

### 2. Edit JSON

Edit only:

- `name` for entries whose `_type` is `name`;
- `message` for normal entries;
- `message_parts` instead of `message` when hidden format controls are present.

Keep `scr_msg`, `_scr_name`, location metadata, raw bytes, and `_format_controls` unchanged.

Name entries contain the clean speaker name. The injector restores the structural `【name】` framing automatically.

Ruby annotations such as `[reading]base` and literal `\n` display breaks do not appear in editable fields. They are removed from changed translations. Dynamic placeholders `%I`, `%B`, and `%F` are hidden from editable text and reinserted between `message_parts`.

### 3. Rebuild BIN files

```powershell
ail_text_tool.exe inject `
  --source <ORIGINAL_BIN_DIR> `
  --json <TRANSLATION_JSON_DIR> `
  --output <REBUILT_BIN_DIR>
```

The injector:

- validates JSON against the matching original BIN;
- preserves index and bytecode lengths;
- changes only confirmed string operands;
- keeps voice opcode `0x48` and all non-text operands byte-for-byte;
- retains the original pool for unknown/unlisted references and appends only final translated strings;
- updates the declared text-pool size;
- rejects CP932 encoding errors and pools larger than 65,535 bytes;
- never truncates translated text.

If every editable field is unchanged, the output BIN is byte-identical to its source.

### 4. Repack SNL

```powershell
ail_text_tool.exe pack-snl `
  --source <ORIGINAL_SNL> `
  --bins <REBUILT_BIN_DIR> `
  --output <NEW_SNL>
```

Unchanged archive entries retain their original compressed bytes. Changed entries are recompressed with an engine-compatible literal LZSS stream.

To unpack an archive:

```powershell
ail_text_tool.exe unpack-snl `
  --input <SNL_FILE> `
  --output <BIN_DIR>
```

## Refresh clean source JSON

After a reference map exists, clean source JSON can be regenerated without consulting the legacy translated BINs:

```powershell
ail_text_tool.exe extract `
  --input <ORIGINAL_BIN_DIR> `
  --references <EXISTING_JSON_DIR> `
  --output <CLEAN_JSON_DIR>
```

The reference JSON acts as the project IR. The tool does not use unsafe every-byte opcode scanning.

## JSON fields

- `_file`, `_index`: source identity and stable row order.
- `_inst_offset`, `_ref_offset`, `_target`, `_opcode`: exact bytecode reference metadata.
- `_type`: `name`, `message`, `choice`, `choice_prompt`, `route_label`, or `system`.
- `_raw_hex`, `_encoding`: source validation data.
- `_scr_name` / `name`: original and writable speaker name.
- `scr_msg` / `message`: original and writable body text.
- `scr_msg_parts` / `message_parts`: text fragments surrounding hidden formatting tokens.
- `_format_controls`: ordered structural placeholders; do not edit.

The reader accepts the obsolete `scr-msg` key as an input alias, but new JSON always writes `scr_msg`.

## Safety and limitations

- Use matching original BINs as `--source`; injected or unrelated builds are rejected.
- JSON is UTF-8; game strings are encoded as CP932.
- Outputs are refused when they already exist unless `--overwrite` is supplied.
- Text reference discovery for this project is anchored by the recovered reference IR. Full semantics for every possible AIL opcode are outside this tool's scope.
- Always keep the original SNL and EXE unchanged and test a separately named rebuilt archive first.

