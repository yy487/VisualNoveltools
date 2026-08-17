# tongern_ks

`tongern_ks` extracts and injects story text from the Tongern KAG 3.20 `.ks`
scripts used by `EDEN`.

## Build

```powershell
cargo build --release --offline --bins
```

## Usage

```powershell
tongern_ks.exe extract "<SCENARIO_DIR>" `
  -o "<OUTPUT_DIR>\scenario_json"
tongern_ks.exe inject "<OUTPUT_DIR>\scenario_json" `
  --source "<SCENARIO_DIR>" -o "<OUTPUT_DIR>\scenario_injected"
```

Dropping a `scenario` directory extracts it. Dropping its JSON directory
injects it using the source recorded in `audit.json`. Output directories must
not already exist. Use `--macro` only when `macro.ks` is outside the source
directory.

## Translation JSON

For a normal entry, edit `message`. If `_controls` is present, edit
`message_parts` instead and keep the number of parts unchanged. `message` must
remain unchanged or equal the joined translated parts.

A displayed `name` may be translated. Keep `_scr_name` unchanged. One speaker
macro must use the same translated name everywhere; the injector updates its
definition in `macro.ks`.

Keep `scr_msg` and every field beginning with `_` unchanged. Translator text
does not include physical newlines, KAG tags, or ruby readings. Hidden controls
are restored by the injector.

## Repairing JSON

`repair-json` rebuilds damaged metadata, repairs unescaped quotes, splits an
edited `message` back into the original `message_parts`, and can apply a name
dictionary:

```powershell
tongern_ks.exe repair-json "<TRANSLATION_DIR>" `
  --source "<SCENARIO_DIR>" --name-dictionary "<NAME_DICTIONARY>" `
  -o "<OUTPUT_DIR>\scenario_json_repaired"
```

The repaired directory must still pass normal injection. Names imported from a
normal Unicode dictionary must first use the same CP932 carrier mapping as the
project's translated text.

## Limits

Injected scripts remain CP932. Characters outside CP932 require the project's
carrier mapping. XP3 packing is not included. The injector does not currently
restore the source line ending after a horizontal-only `[locate x=...]` control;
any manual correction to that case is overwritten by reinjection.
