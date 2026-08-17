# Waka DAT and TLG tools

These tools handle ACV1/NonColor archives and images from
`和香様の座する世界`.

- `waka_dat_tool.py` unpacks, verifies, and rebuilds `arc*.dat` and
  `script*.dat` archives.
- `nc_filemap_tool.py` builds filename lists from GARbro's NonColor map.
- `tlg2png/tlg2png.exe` converts TLG images to PNG.

## DAT usage

```powershell
python waka_dat_tool.py unpack <ARC_DAT> <WORK_DIR> --mode arc --name-list <NAME_LIST>
python waka_dat_tool.py unpack <SCRIPT_DAT> <WORK_DIR> --mode script
python waka_dat_tool.py verify <SOURCE_DAT> <WORK_DIR>
python waka_dat_tool.py pack <WORK_DIR> <OUTPUT_DAT>
```

The unpacked directory contains `manifest.json`; keep it with `files/` for
verification and packing.

## Filename map

```powershell
python nc_filemap_tool.py --idx <NCFILEMAP.IDX> --dat <NCFILEMAP.DAT> export <ARC_DAT> -o <NAME_LIST> --prefix z/
```

## TLG conversion

```powershell
tlg2png\tlg2png.exe <INPUT.TLG> <OUTPUT.PNG>
powershell -ExecutionPolicy Bypass -File Convert-AllTlgToPng.ps1
```

## Limits

- Unknown ARC hashes are extracted under generated names unless a matching
  filename list is supplied.
- `--only-named` creates an incomplete workspace for inspection or image
  conversion; do not use it as input to `pack`.
- The bundled image tool converts TLG to PNG only.
