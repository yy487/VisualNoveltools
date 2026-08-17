# AVG32 FN.DAT font tool

`fn.exe` redraws the 24x24 bitmap font used by AVG32 and applies the embedded
Chinese carrier-character mapping.

Place `fn.exe` and the original `FN.DAT` in the same directory, then run:

```powershell
fn.exe "<FONT_FILE>"
```

The font may be TTF, TTC, or OTF. With no argument, the program asks for a font
and can select the first supported font in its directory. The output is
`fn_chs.dat` beside the program.

Mapped carrier slots are redrawn as Chinese characters. Other valid slots are
redrawn as their original JIS characters. If the selected font lacks a glyph,
that slot is copied from the original `FN.DAT`.

## Limits

The input must use this game's fixed AVG32 `FN.DAT` layout. Only BMP Unicode
characters are supported. A TTC uses its first font face. Existing
`fn_chs.dat` is not overwritten.
