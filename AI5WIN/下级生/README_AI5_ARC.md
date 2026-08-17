# AI5WIN ARC extractor

`ai5win_arc_extract.py` lists or extracts the ARC format used by this version
of `下級生`.

## Usage

```powershell
python ai5win_arc_extract.py list <ARCHIVE.ARC>
python ai5win_arc_extract.py extract <ARCHIVE.ARC> -o <OUTPUT_DIR>
python ai5win_arc_extract.py extract <A.ARC> <B.ARC> -o <OUTPUT_DIR> --overwrite
```

Use the global `--encoding` option before the command when archive names are
not CP932.

## Limits

- This is an extractor only; it does not rebuild ARC files.
- It supports the count-keyed, permuted 20-byte directory used by the target
  game. Other AI5WIN ARC layouts need their own tools.
- Archive members are written as stored. The extractor does not decode member
  formats such as MES scripts or images.
