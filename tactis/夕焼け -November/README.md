# Yuyake MES text tools

`extract.exe` and `inject.exe` handle the extensionless MES scripts used by
`夕焼け -November-`.

## Usage

```powershell
extract.exe <MES_FILE_OR_DIR> --output <JSON_FILE_OR_DIR>
inject.exe <MES_FILE_OR_DIR> <JSON_FILE_OR_DIR> --output <OUTPUT_FILE_OR_DIR>
```

Directory injection copies all source files and replaces only scripts with a
matching JSON file.

Edit only `message`. Keep `scr_msg` and fields beginning with `_` unchanged.
This project has no `name` field. Do not add CR/LF or edit the control layout.

## Limits

- Injection is in-place: each encoded message must fit its original page slot
  and the file size cannot change.
- Shorter text is padded with ignored `03 03` control pairs, so the unused
  byte count must be even.
- Text must be CP932. Instructions, jumps, sentence counts, and controls cannot
  be added, removed, or relocated.
