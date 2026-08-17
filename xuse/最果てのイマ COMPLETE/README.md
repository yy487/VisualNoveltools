# Farthest2015 CD localization tool

This Rust tool is used to verify, extract, and rebuild the `.cd` scripts from
Farthest2015 COMPLETE.

It extracts messages, names, choices, and hyperlinks to UTF-8 JSON. The default
script encoding is CP932.

## Build

```powershell
cargo build --release --offline
```

## Usage

Verify scripts:

```powershell
farthest2015-cd-tool verify --source "<GAME_DIR>" --text-encoding cp932
```

Extract JSON:

```powershell
farthest2015-cd-tool extract --source "<GAME_DIR>" `
  --output "<OUTPUT_DIR>\json" --text-encoding cp932
```

Write translations back:

```powershell
farthest2015-cd-tool inject --source "<GAME_DIR>" `
  --translation "<OUTPUT_DIR>\json" --output "<OUTPUT_DIR>\scripts"
```

Existing output requires `--overwrite`. With no arguments, the program opens a
menu. Dropped paths are editable prefills and do not write before confirmation.

## Translation JSON

Edit `message`. If `name` is present, it may also be translated. Do not edit
`scr_msg`, `_scr_name`, or fields beginning with `_`.

Messages are stored as one string even when the source used several visual
lines. The game wraps text automatically, so original visual line breaks are
not placed in `message`. Add an LF only when a forced line break is wanted.

Clickable text uses:

```text
[[link:0]]clickable text[[/link]]
```

The tags are not displayed. They are used to rebuild the link range. A complete
tag pair may be removed to disable that link. Malformed, crossing, empty, or
duplicate link tags are rejected.

## Encoding

CP932 is the default. Use `--target-encoding gbk` only with a game executable
already patched to read CP936. This tool does not patch the executable or font.

Each encoded string must fit the runtime's 2,044-byte limit.

## Limits

The tool handles the profiled scenario, label, and variable files. Changed text
may disable ruby or range-based font controls that cannot be rebuilt safely; the
program reports these cases. Use a game copy for final display, link, choice,
save, and load testing.
