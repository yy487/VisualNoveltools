# Noesis / Love es M localization tools

These Python tools handle the Noesis `script.iga` archive and its `.s` scripts
for `Love es M`.

- `noesis_unpack.py` extracts `script.iga`.
- `noesis_pack.py` rebuilds `script.iga`.
- `noesis_extract.py` extracts script text to UTF-8 JSON.
- `noesis_inject.py` writes translated JSON back with variable-length
  relocation.
- `noesis_pipeline.py` exposes the same operations as subcommands.

## Usage

```powershell
python noesis_unpack.py "<SCRIPT.IGA>" "<SCRIPT_DIR>"
python noesis_extract.py "<SCRIPT_DIR>" "<JSON_DIR>"
python noesis_inject.py "<SCRIPT_DIR>" "<JSON_DIR>" "<INJECTED_DIR>"
python noesis_pack.py "<INJECTED_DIR>" "<OUTPUT.IGA>"
```

By default, extraction removes ruby markup from the initial `message`. Add
`--keep-ruby` when the editable text should keep the original ruby syntax.

Use `--output-encoding gbk` on `noesis_inject.py` only when the game has a
matching GBK text hook and font:

```powershell
python noesis_inject.py "<SCRIPT_DIR>" "<JSON_DIR>" "<INJECTED_DIR>" --output-encoding gbk
```

## Translation JSON

```json
{
  "name": "Translated name",
  "scr_msg": "Original message",
  "message": "Translated message",
  "_file": "0010.s",
  "_index": 9,
  "_type": "dialogue"
}
```

Edit `message` and, for dialogue entries, `name`. Do not edit `scr_msg` or fields
beginning with `_`.

Directory extraction also creates `_noesis_name_dict.json`. Edit its values to
apply one speaker-name translation across all scripts:

```json
{
  "あやか": "绫香"
}
```

The injector also accepts `--name-dict <FILE>` to use an explicit dictionary.

## Limits

- Ordinary text and name records use a one-byte length. Encoded text plus its
  terminator must not exceed 255 bytes.
- Choice text may not exceed 65,535 encoded bytes.
- Variable-length injection updates only the confirmed `1D08`, `0D08`, and
  `3B08` physical offsets.
- CP932 is the default output encoding. GBK/GB18030 output only changes script
  bytes and still requires compatible runtime and font support.
- One original speaker name cannot map to several translations.
- Resource, voice, image, BGM, and sound-effect names are not translated.
