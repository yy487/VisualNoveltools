# Asoko no Shiawase FREECG98 font tool

`aitsuno_freecg98_tool` creates the Simplified Chinese `FREECG98.BMP` used with
the text tool's built-in CP932 carrier mapping. The base image, mapping, and
16x16 glyphs are embedded in the executable.

## Build

```powershell
cargo build --release --bin aitsuno_freecg98_tool
```

## Usage

Create the font image:

```powershell
aitsuno_freecg98_tool.exe render --output "<OUTPUT.BMP>"
```

Check an existing generated image:

```powershell
aitsuno_freecg98_tool.exe verify --input "<OUTPUT.BMP>"
```

Add `--overwrite` to replace an existing render output. Running the tool with no
arguments opens the interactive menu; passing one path prefills the render
output.

## Limits

- Glyphs are fixed 16x16 monochrome bitmaps.
- Characters outside the built-in mapping are not assigned automatically.
- The tool only creates and verifies `FREECG98.BMP`; it does not edit scripts or
  disk images.
