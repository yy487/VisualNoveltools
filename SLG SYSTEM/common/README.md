# SZS100__ archive tool

`szs_auto.py` lists, decrypts, unpacks, and rebuilds `SZS100__` archives used
by several SLG SYSTEM games.

## Usage

```powershell
python szs_auto.py list <ARCHIVE.SZS>
python szs_auto.py detect <ARCHIVE.SZS> --exe <GAME_EXE>
python szs_auto.py unpack <ARCHIVE.SZS> <WORK_DIR> --exe <GAME_EXE>
python szs_auto.py pack <WORK_DIR> <OUTPUT.SZS>
```

Unpacking writes `manifest.json`. Keep it with the extracted files because it
stores the archive table and encryption settings used by `pack`.

If automatic detection fails, pass `--seed`, `--xor`, and `--mode` manually.
The supported modes are `full_lcg_sub` and `reseed_lcg_xor`.

## Limits

- Automatic key detection is heuristic and may require the game executable,
  decompiler output, or manual settings.
- Only the two encryption modes above are implemented.
- This tool handles the SZS container and member encryption. It does not
  extract text from the scripts stored inside the archive.
