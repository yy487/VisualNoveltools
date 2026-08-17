# Yuukai system.dat archive tool

`yuukai_dat_tool.py` lists, verifies, unpacks, and rebuilds the raw-deflate
`system.dat` used by `誘拐報道`.

## Usage

```powershell
python yuukai_dat_tool.py list <SYSTEM.DAT> --limit 30
python yuukai_dat_tool.py verify <SYSTEM.DAT>
python yuukai_dat_tool.py unpack <SYSTEM.DAT> <UNPACKED_DIR>
python yuukai_dat_tool.py pack <UNPACKED_DIR> <OUTPUT.DAT>
python yuukai_dat_tool.py roundtrip <SYSTEM.DAT> <WORK_DIR>
```

Keep `_manifest.json` and the `files` directory together. The packer
recompresses each member, rebuilds the AMF3 catalog, and writes its new offset
to the first four bytes of the archive.

## Limits

- Only the AMF3 catalog subset used by this archive is supported.
- Existing members may be replaced, but automatic insertion of new members is
  not implemented.
- Member names and manifest structure should not be changed.
