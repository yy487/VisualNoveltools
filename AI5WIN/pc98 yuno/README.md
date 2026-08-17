# YU-NO PC-98 localization tools

These Rust tools handle the PC-98 version of
`この世の果てで恋を唄う少女 YU-NO`.

- `unpack_hdi` extracts the Anex86 HDI filesystem.
- `pack_hdi` rebuilds an HDI from an unpacked directory.
- `unpack_yuno` extracts the `YUNO_A` through `YUNO_Q` resource archives.
- `pack_yuno` rebuilds those resource archives.
- `extract_mes` extracts MES text to UTF-8 JSON.
- `inject_mes` writes translated JSON back to MES files.
- `unpack_mes`, `pack_mes`, and `verify_mes` are low-level MES analysis tools.

## Build

```powershell
cargo build --release --offline --bins
```

## Main workflow

Extract the HDI:

```powershell
unpack_hdi.exe "<GAME.HDI>" --output "<HDI_DIR>"
```

Extract one or more YUNO archives:

```powershell
unpack_yuno.exe "<HDI_DIR>\YU-NO\YUNO_A" "<HDI_DIR>\YU-NO\YUNO_B" --output "<RESOURCE_DIR>"
```

Extract MES text:

```powershell
extract_mes.exe "<RESOURCE_DIR>" --output "<JSON_DIR>"
```

Inject translated JSON into a new resource tree:

```powershell
inject_mes.exe "<RESOURCE_DIR>" "<JSON_DIR>" --output "<INJECTED_DIR>"
```

Rebuild an archive, place it in a copy of the unpacked HDI tree, then rebuild
the disk image:

```powershell
pack_yuno.exe "<INJECTED_DIR>\YUNO_A" --output "<PACKED_YUNO_A>"
pack_hdi.exe "<GAME.HDI>" "<MODIFIED_HDI_DIR>" --output "<OUTPUT.HDI>"
```

Every output path must be new. Most commands choose a new default path when
`--output` is omitted; use `--help` to see it.

## Translation JSON

Static-name dialogue uses:

```json
{
  "_scr_name": "Original name",
  "name": "Translated name",
  "scr_msg": "Original message",
  "message": "Translated message"
}
```

Edit `message` and, when present, `name`. Do not edit `scr_msg`, `_scr_name`, or
other fields beginning with `_`.

Some messages use `scr_msg_parts` and `message_parts`. Edit the strings in
`message_parts` without changing the number of parts. Entries with a dynamic
player name do not contain `name` and cannot be given one. Do not change
`_message_controls` or `_name_controls`.

## Low-level MES commands

```powershell
unpack_mes.exe "<SCRIPT.MES>"
pack_mes.exe "<SCRIPT.MES.decoded>"
verify_mes.exe "<MES_FILE_OR_DIR>"
```

These commands are for compression and format checks. Use `extract_mes` and
`inject_mes` for translation.

## Limits

- HDI support is limited to the FAT16 layout used by this game. Files may
  change size, but filesystem paths cannot be added, removed, or renamed.
- Archive tools accept only `YUNO_A` through `YUNO_Q`. Resources may change
  size, but each stored resource must remain at most 65,535 bytes.
- A decoded MES script must remain at most 65,535 bytes.
- Text must use the game's dictionary or supported double-byte CP932 tokens.
  Unsupported single-byte text, NUL, and line breaks are rejected.
- Chinese text requires a separate character mapping and matching font work.
