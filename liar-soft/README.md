# Seven Bridges GSC localization tools

This directory contains two Windows tools for `Seven Bridges`.

- `wcg-png` converts Liar-soft WCG images to RGBA PNG.
- `sbridge-gsc` extracts GSC text to JSON and writes translations back.

## WCG images

```powershell
wcg-png.exe convert "<GAME_DIR>\image.wcg" `
  --output "<OUTPUT_DIR>\image.png" --yes
wcg-png.exe convert "<GAME_DIR>\graphics" `
  --output "<OUTPUT_DIR>\png" --recursive --yes
```

Use `--overwrite` to replace existing PNG files. PNG-to-WCG conversion is not
supported.

## GSC scripts

```powershell
sbridge-gsc.exe extract "<GAME_DIR>\scripts" `
  --output "<OUTPUT_DIR>\translation_json" `
  --speaker-map "data\speaker_map.json" --yes
sbridge-gsc.exe inject "<OUTPUT_DIR>\translation_json" `
  --source "<GAME_DIR>\scripts" `
  --output "<OUTPUT_DIR>\scripts_rebuilt" --yes
```

Both tools also have an interactive menu. A dropped path prefills that menu;
writing still requires confirmation.

## Translation JSON

Edit only `message`. `scr_msg` is the original text. `name` is display context
from a WCG nameplate and is not written to the script. Fields beginning with `_`
must not be edited.

Original `^n` line breaks are hidden because the game wraps text automatically.
An unchanged message keeps its original bytes. Add a real line break or valid
`^n` only when a forced break is wanted. The `^gNNN` speaker-nameplate resource
is always kept from the source.

## Limits

Text must be encodable as CP932. WCG conversion is one-way. Rebuilding the outer
XFL archive and patching fonts are not included, so rebuilt GSC files still need
to be installed and tested in a game copy.
