# Triple Pairing BGI text tools

These Python tools extract and inject dialogue, choices, and selected UI text
from BGI V1 scripts or their `.bsd` disassembly.

## Usage

```powershell
python extract.py <SCRIPT_OR_BSD_DIR> <JSON_DIR> --mode auto
python inject.py <SCRIPT_OR_BSD_DIR> <JSON_DIR> <OUTPUT_DIR> --mode auto
```

Use `--mode script` for compiled scripts and `--mode bsd` for disassembly.
Script mode accepts `--encoding shift_jis --fallback-encoding gbk`.

To compare all disassembled strings with the exported JSON:

```powershell
python audit_bsd_strings.py <INPUT_DIR> <JSON_DIR> <REPORT.JSON> --mode auto
```

## Translation JSON

Edit `message`. A present `name` may also be translated. Keep `scr_msg` and
fields beginning with `_` unchanged. Choices use `_type: "choice"`; selected
chapter and window strings use `_type: "ui"`.

## Limits

- The extractor recognizes the documented BGI V1 dialogue and choice call
  patterns; the audit command should be used to find unclassified strings.
- Compiled-script injection depends on successful disassembly and assembly.
- Encoding and font support must match the target game.
