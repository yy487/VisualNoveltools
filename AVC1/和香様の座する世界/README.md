# 和香 DAT/TLG tools

This folder collects the tools used for `和香様の座する世界` ACV1/NonColor archives.

## Files

- `waka_dat_tool.py`
  - Unpacks ACV1 `arc*.dat` and `script*.dat`.
  - Supports GARbro-compatible CRC64 filename maps.
  - Use `--name-list` for stable real paths such as `z/ev/*.tlg`, `z/st/*`, `z/sys/*`.

- `nc_filemap_tool.py`
  - Reads GARbro `GameData/NCFileMap.dat` + `.idx`.
  - Exports names whose CRC64 hashes match this game's archive entries.

- `name_lists/waka_z_names.txt`
  - Main usable name list for this game.
  - Currently covers `z/an`, `z/bg`, `z/cg`, `z/em`, `z/ev`, `z/st`, and `z/sys`.

- `tlg2png/tlg2png.exe`
  - Converts TLG5/TLG6/TLG0 SDS-wrapped TLG images to PNG.
  - Source is in `tlg2png/Tlg2Png.cs`.

- `Convert-AllTlgToPng.ps1`
  - Batch converts extracted `.tlg` files into `pic/`, preserving the resource folder layout.

- `references/`
  - GARbro reference source files used to confirm the archive and TLG formats.

## Common Commands

Run these from this project directory.

Extract named `z/` resources:

```powershell
python tools\waka_dat_tool.py unpack arc3.dat arc3_z_named --mode arc --name-list tools\name_lists\waka_z_names.txt --only-named
python tools\waka_dat_tool.py unpack arc2.dat arc2_z_named --mode arc --name-list tools\name_lists\waka_z_names.txt --only-named
```

Convert one TLG:

```powershell
tools\tlg2png\tlg2png.exe arc3_z_named\files\z\ev\EV_001_00_0A.tlg pic\z\ev\EV_001_00_0A.png
```

Convert all extracted TLG files into `pic/`:

```powershell
powershell -ExecutionPolicy Bypass -File tools\Convert-AllTlgToPng.ps1
```

Export matching names again from GARbro's `NCFileMap`:

```powershell
python tools\nc_filemap_tool.py export arc0.dat arc1.dat arc2.dat arc3.dat -o tools\name_lists\nc_z_matched_names.txt --prefix z/
```

The GARbro file map files are under its installation directory:

```text
<GARbro_DIR>\GameData\NCFileMap.dat
<GARbro_DIR>\GameData\NCFileMap.idx
```
