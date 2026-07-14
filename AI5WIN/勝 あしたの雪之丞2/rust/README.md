# AI5WIN ARC/G24/MSK tools

This directory contains the Rust replacement for the original Python archive and image tools.
The executable supports explicit commands and Windows drag-and-drop.

## Build

```powershell
cargo build --release
```

The executable is `target/release/ai5win-tools.exe`.

## Commands

```powershell
ai5win-tools arc-unpack Bg.arc bg_unpacked
ai5win-tools arc-pack bg_unpacked rebuilt.arc
ai5win-tools arc-verify Bg.arc rebuilt.arc
ai5win-tools g24-decode input.G24 output.png
ai5win-tools g24-encode input.png output.G24 --reference original.G24
ai5win-tools msk-decode input.MSK output.png --width 640 --height 480
ai5win-tools msk-encode input_M.png output.MSK --kind raw
ai5win-tools msk-encode title.png TITLE_PT_M.MSK --kind title-pt
ai5win-tools build-patch --bg Bg.arc --data data.arc --edits "修图" --output .
```

Dragging an ARC onto the executable unpacks it. Dragging a G24 decodes it. Dragging a PNG
encodes it as G24, or as raw MSK when its stem ends in `_M`. Dragging the repair directory
builds `bg_chs.arc` and `data_chs.arc` when `Bg.arc` and `data.arc` are beside that directory.
Headerless MSK files other than `TITLE_PT_M.MSK` require explicit dimensions when decoded.

## Formats and validation

- ARC: encrypted 32-byte entries, original order retained. Patch builds stream-copy every
  unchanged entry and verify every rebuilt entry against either the original bytes or the
  replacement bytes.
- G24: `<i16 x, i16 y, i16 width, i16 height>`, followed by Okumura LZSS. Pixels are 24-bit
  BGR in bottom-up DIB order with 4-byte row alignment. Patch builds preserve the original
  x/y values and require unchanged dimensions.
- MSK Type A: decompressed `<u16 width, u16 height>` plus `width*height` values in 0..16.
- MSK raw: headerless row-major 8-bit values. Patch builds infer dimensions and type from the
  original entry and the edited PNG.
- LZSS: 4096-byte ring, initial position `0xFEE`, 3..18 byte matches.

## TITLE_PT_M disassembly basis

The special layout is supported because the executable and script data flow agree, not because
of an image-only guess:

1. Decompressed `TITLE2.MES` offset `0x96A4` loads `title_pt.g24`. Offset `0x96B6` invokes
   `MENU_SET 0x54` with `title_pt_m.msk` and expression `05 F5 00` (buffer 5).
2. `Ai5win.exe` case 84 at `0x40E1F9..0x40E26D` copies the explicit filename and calls
   `sub_4042B0`, placing the decompressed bytes in that expression result.
3. `TITLE2.MES` offsets `0x87C4`, `0x8841`, `0x88A6`, `0x8907`, `0x8988`, `0x89ED`, and
   `0x8A4E` invoke `MENU_SET 0x1B` (case 27). In every call the mask pointer is `05 F5 00`
   and the final argument is `F1 70 02 FF`, the constant 624.
4. Case 27 at `0x40D2B0..0x40D355` passes that final argument as `a15` to `sub_40B4E0`.
   The function starts at `mask + a12 + a15*a13`, increments one byte for each x pixel,
   and adds `a15` at each row. The addressing is therefore `mask[y*624+x]`.
5. The decompressed payload is 361,920 bytes. `361920 / 624 = 580`, proving one physical
   624x580 row-major image. Three 208x580 views may only be cropped along x after decoding;
   they are not three sequential frames in the file.

The relevant exported files are `Ai5win.exe_export_for_ai/disassembly/40C570.asm` and
`Ai5win.exe_export_for_ai/decompile/40B4E0.c`.

## Patch policy and known limits

`build-patch` maps `INTRO_CBG1.png` to `INTRO_CBG.G24`. `INTRO_CBG.png` is retained as a
1448x1086 source canvas and is not packed. `TITLE_PT_M_full_624x580.png` maps to
`TITLE_PT_M.MSK`. Unknown entries are never regenerated; they are copied byte-for-byte.
The tool does not guess dimensions for arbitrary headerless MSK files.
