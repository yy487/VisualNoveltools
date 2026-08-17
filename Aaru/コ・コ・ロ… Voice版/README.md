# KOKOROV FL2 / AB localization tools

These tools are used to extract and translate KOKOROV games.

- `fl2_unpack` extracts an FL2 archive.
- `fl2_pack` rebuilds an FL2 archive.
- `ab_inspect` checks an AB script.
- `ab_extract` extracts AB text to JSON.
- `ab_inject` writes translated text back to AB scripts.

## Build

```powershell
cargo build --release --bins
```

The programs are created in `target\release`.

## FL2 archives

```powershell
fl2_unpack.exe "<GAME_DIR>\A.FL2"
fl2_unpack.exe "<GAME_DIR>\A.FL2" --output "<OUTPUT_DIR>\A_unpacked"
fl2_pack.exe "<OUTPUT_DIR>\A_unpacked" --output "<OUTPUT_DIR>\A_packed.FL2"
```

Keep `fl2_manifest.json` in the extracted directory. It is required for
packing. Use `--overwrite` to replace an existing output.

## AB scripts

```powershell
ab_extract.exe "<GAME_DIR>\mes" --output "<OUTPUT_DIR>\mes_json"
ab_inject.exe "<GAME_DIR>\mes" "<OUTPUT_DIR>\mes_json" `
  --output "<OUTPUT_DIR>\mes_injected"
```

The same commands also accept one AB file and one JSON file. Directory
injection copies the full source tree and replaces only translated scripts.

## Translation JSON

Edit `message`. If `name` is present, it may also be translated. Do not edit
`scr_msg`, `_scr_name`, or fields beginning with `_`.

Text must be encodable as CP932. Messages and names may be shorter or longer
than the source. `\W` and `%M0` are editable message controls. `\N` separates
the speaker name from the message and cannot be used as body text.

## Limits

Only the supported KOKOROV FL2 and AB layouts are accepted. FL2 entries cannot
be added, removed, or renamed. Unknown AB instructions are errors. Font and
encoding patches for characters outside CP932 are not included.
