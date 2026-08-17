# Floreal SEEN.TXT text tools

These Rust tools extract and inject text from the AVG3216 `SEEN.TXT` used by
`フロレアール ～すきすきだいすき～`.

- `avg3216_extract` writes UTF-8 JSON.
- `avg3216_inject` rebuilds `SEEN.TXT` from translated JSON.

## Build

```powershell
cargo build --release --offline --bins
```

## Usage

```powershell
avg3216_extract.exe --no-pause --output <TEXT.JSON> <SEEN.TXT>
avg3216_inject.exe --no-pause --source <SEEN.TXT> --output <OUTPUT.TXT> <TEXT.JSON>
```

The programs also support Windows drag and drop.

## Translation JSON

Edit only `message`. Keep `scr_msg` and fields beginning with `_` unchanged.
This project has no `name` field. Choice entries also use `message`.

Every encoded character must be a two-byte CP932 character; half-width ASCII,
digits, spaces, and punctuation are rejected. Messages may grow or shrink, and
the tool rebuilds the affected script and archive tables.

## Limits

- Only this game's profiled AVG3216 layout is supported.
- Output remains CP932. Chinese text requires a carrier mapping and matching
  `FN.DAT` font.
- The tool does not create the executable encoding patch or font mapping.
