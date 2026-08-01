# tongern_ks

Structure-aware story-text extractor and injector for the Tongern KAG 3.20
`.ks` scripts.

The tool validates source scripts as byte-exact CP932 and writes one UTF-8 JSON
file per source script. It extracts story dialogue, narration, choices, and
speaker display-name definitions. Menu and other UI text are excluded.

## Build

```powershell
cargo build --release --offline --bins
```

The executable is `target\release\tongern_ks.exe`.

## Drag and drop

Drag the whole `scenario` directory onto `tongern_ks.exe`:

```text
scenario\ -> scenario_json\
```

Translate the JSON files, then drag `scenario_json` onto the EXE:

```text
scenario_json\ -> scenario_injected\
```

The injector reads the original source path from `audit.json`. Output
directories must not already exist. The original game directory is never
overwritten.

## Command line

```powershell
.\target\release\tongern_ks.exe extract `
  E:\GAL\tongern\ks\scenario `
  -o H:\IDA-PRO-MCP\tongern\analysis_output\scenario_json

.\target\release\tongern_ks.exe inject `
  H:\IDA-PRO-MCP\tongern\analysis_output\scenario_json `
  --source E:\GAL\tongern\ks\scenario `
  -o H:\IDA-PRO-MCP\tongern\analysis_output\scenario_injected

.\target\release\tongern_ks.exe repair-json `
  E:\GAL\tongern\ks\scenario_json `
  --source E:\GAL\tongern\ks\scenario `
  --name-dictionary E:\GAL\字典\终结世界\终结世界_name.toml `
  -o H:\IDA-PRO-MCP\tongern\analysis_output\scenario_json_repaired
```

Use `--macro PATH` only when `macro.ks` is not inside the source directory.
Run `tongern_ks.exe --help` for the complete syntax.

## JSON contract

Translator fields:

```text
name          Optional writable displayed speaker name.
scr_msg       Immutable source story text; never written back.
message       Editable translation for entries without protected controls.
```

Named dialogue also contains immutable `_scr_name`. A changed `name` is
validated and written once through its `_speaker_macro` definition in
`macro.ks`; all occurrences of one speaker macro must use the same translated
name.

Translator text contains only story text. Physical CR/LF, `[l]`, `[r]`, ruby
tags, and all other KAG tags are absent.

When `_controls` exists, the entry also has:

```text
scr_msg_parts       Immutable source text slots.
message_parts       Editable translation slots used for injection.
_controls           Immutable hidden KAG controls restored by the injector.
```

For these multipart entries, edit `message_parts`. Editing only `message` is
rejected. It is valid to leave `message` unchanged and edit only
`message_parts`; if both are edited, `message` must equal the concatenated
parts. The number of parts cannot change.

All underscore-prefixed location and structure fields are immutable. This
includes `_file`, `_index`, `_line`, `_end_line`, `_offset`, `_size`, `_type`,
`_encoding`, `_boundary`, `_source_kind`, `_speaker_macro`, `_target`,
`_ruby_removed`, `_controls`, and `_scr_name`.

## Repair translated JSON

`repair-json` is for translation programs that edit only `message`, modify
immutable fields, or emit unescaped quotes. It:

- reparses the original scripts and rebuilds all immutable metadata;
- retains translated `message` strings even when inner ASCII quotes were not
  escaped;
- deterministically splits translated text back into the original number of
  `message_parts`, preferring nearby matching punctuation;
- applies the supplied name dictionary globally through speaker macros;
- preserves the original display name when the dictionary has no entry;
- writes a new UTF-8 JSON directory and never overwrites its input.

The repaired directory must still pass normal injection validation. Repairing
JSON does not make Unicode characters representable in CP932.

This project's translated JSON may already contain CP932 carrier characters
used by UIF `character_substitution`. The injector writes those characters as
provided and does not apply or reverse the font mapping. Values newly imported
from a normal-Unicode name dictionary must be converted to the same carrier
form before injection.

## Control policy

- Physical source newlines are omitted from translator text so translated
  story text can use the message layer's `autoReturn=true` soft wrapping.
- A physical-line-final `[l]` is hidden in `_controls` and restored
  automatically when that entry is modified.
- An inline `[l]` is discarded from the rebuilt translated entry.
- Every `[r]` is discarded from the rebuilt translated entry.
- Other inline KAG effects and waits are hidden in `_controls` and restored in
  their original order between `message_parts`.
- `[ruby text=...]` is deleted while its following base text is retained.
- Unchanged entries are not rewritten at all, which is required for the
  byte-exact no-change round trip.
- A translated multipart entry with a new physical source line beginning with
  `[locate x=...]` and no `y` needs that source line ending restored in the
  final injected script. The horizontal-only `locate` resets `x` but retains
  `y`, so flattening this boundary can draw the next line over the previous
  one. Do not apply this rule to `locate` tags that explicitly set `y`.

## Injection validation

Before creating an output directory, the injector:

- reparses the source with the same state machine;
- validates every immutable JSON field and entry count;
- rejects NUL, CR/LF, KAG square-bracket syntax, and unsafe line syntax in
  edited translator fields;
- strictly encodes every replacement as CP932 and lists unencodable Unicode
  characters;
- rejects missing, extra, reordered, or structurally changed entries;
- applies byte patches by `_offset` in descending order.

After validation, the complete source tree is copied to a new directory and
only modified script spans are replaced. A failure does not leave a partial
final output directory.

## Extraction policy

- Speaker definitions are discovered from the structure of `macro.ks`.
- `[p]`, `[er]`, `[cm]`, `[ct]`, and speaker macros drive the message state
  machine.
- Dialogue `name` is the displayed macro name and is not written per dialogue.
- Speaker display names are extracted once from `macro.ks` as `_type: name`
  entries. Dialogue `name` changes and these definitions are synchronized by
  `_speaker_macro`.
- Story link bodies in `7th.ks` are extracted as choices.
- UI-only scripts, menu regions, UI links, visible tag attributes, resource
  names, and TJS string literals are excluded.
- `first2nd.ks` extracts messages only between the confirmed `*0001` story
  start and `*config3` UI start labels.

## Verified source profile

The source directory contains 22 byte-exact CP932 `.ks` files. The established
story profile contains 15,160 entries:

```text
dialogue   9,016
monologue  6,053
choice        11
name          80
ui             0
```

The extractor removes 1,266 ruby readings. Four source violations are known:
stray `]` after valid `[p]` tags at `3rd.ks:1759`, `7th.ks:11722`,
`8th.ks:3581`, and `8th.ks:5217`.

## Encoding and limitations

JSON is UTF-8. Injected `.ks` files remain strict CP932. This exact Kirikiri
2.26.3 runtime has not passed a controlled UTF-8 scenario test, so the tool
does not silently switch scripts to UTF-8 or UTF-16. Chinese characters not
representable in CP932 must already use the project's UIF carrier mapping or
they are rejected.

The tool does not pack XP3 archives or build the planned `patch2.xp3`.
The injector does not currently restore the horizontal-only `locate` line
endings described above; reinjection therefore overwrites any manual fixes.
