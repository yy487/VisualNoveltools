# XAF Script.IDA archive tool

`ida_xaf_tool.py` lists, unpacks, verifies, and rebuilds the `Script.IDA`
archive used by `ファムファタル`.

## Usage

```powershell
python ida_xaf_tool.py list <SCRIPT.IDA>
python ida_xaf_tool.py unpack <SCRIPT.IDA> <UNPACKED_DIR>
python ida_xaf_tool.py verify <SCRIPT.IDA> <UNPACKED_DIR>
python ida_xaf_tool.py pack <UNPACKED_DIR> <OUTPUT.IDA>
python ida_xaf_tool.py roundtrip <SCRIPT.IDA> <WORK_DIR>
```

Keep `_ida_xaf_manifest.json` with the extracted `files` directory; it contains
the archive records required for packing.

## Limits

- Supports the profiled `XAF\0` archive and its NOT, chained XOR, chained
  ADD/SUB, zlib, and legacy RLE payload flags.
- Repacking may produce different compressed bytes.
- This tool handles the archive only; member formats are not translated.
