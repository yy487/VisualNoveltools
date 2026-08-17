# diannao_ks

`diannao_ks` extracts and injects story text from the game's KAG `.ks` scripts.

## Build

```powershell
cargo build --release --offline --bins
```

## Usage

```powershell
diannao_ks.exe extract "<SCENARIO_DIR>" -o "<OUTPUT_DIR>\scenario_json"
diannao_ks.exe inject "<OUTPUT_DIR>\scenario_json" `
  --source "<SCENARIO_DIR>" -o "<OUTPUT_DIR>\scenario_injected"
```

Dropping a `scenario` directory extracts it. Dropping its JSON directory
injects it using the source recorded in `audit.json`. Output directories must
not already exist.

## Translation JSON

Edit `message`. Keep `scr_msg` and all fields beginning with `_` unchanged.
This project has no writable speaker-name field. Choices also use `message`.

`_display_prefix` and `_display_suffix` show protected macro output such as
`[dash]`; they must remain unchanged. Dynamic `[emb]` expressions are also
read-only. Do not add NUL, real CR/LF, or KAG square-bracket tags to a message.
Existing script tags and controls remain outside the edited text spans.

Text must be encodable as CP932. Plain-text messages may grow or shrink.

## Limits

This tool handles unpacked `.ks` files only; it does not rebuild an outer
archive or patch fonts. Conditional branches are extracted in source order,
not simulated. Dynamic `[emb]` values cannot be expanded in static JSON.
