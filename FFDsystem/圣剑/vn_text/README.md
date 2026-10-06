# vn-text

`vn-text` defines the translator-facing JSON view used for secondary extraction.
The game's primary extraction is adapter-owned: its structure and JSON format
can vary without changing this library. An adapter projects its editable fields
into this template, then applies validated edits back to its own full records.

## Translation JSON

Each source unit is represented by a top-level JSON array. Every item has a
required `message`; `name` is optional and appears only when the adapter's
translation policy includes a name for that entry:

```json
[
  { "name": "Speaker", "message": "Translated line" },
  { "message": "Nameless line" }
]
```

This matches the translator-facing JSON projection in `megas`: no source hash,
offset, token data, or other primary-extraction metadata is copied into this
view. Unknown fields are rejected when the JSON is read. An absent `name` is
omitted on write; an explicit JSON `null` is read as absent, matching Rust's
optional-field behavior.

## New project integration

Add the shared crates to a game adapter beside the other Rust projects:

```toml
[dependencies]
vn-cli = { path = "../vn_cli" }
vn-text = { path = "../vn_text" }
```

Keep generated project artifacts in `projects/rust/<tool>/work/<game>/` (for
example, separate `primary/`, `translations/`, and `rebuilt/` subdirectories).
Do not create a shared `work/` directory at the repository root. Register the
new adapter path in `workspace_manifest.json`.

Keep parsing, source identity, locators, encoding, controls, and binary rebuild
in the game adapter. The full primary records may stay in memory or use any
game-specific representation. The export operation projects those records to
`Vec<vn_text::Entry>` and calls `write_template`; the import operation reloads
the current source and builds the same expected projection, then follows this
sequence:

```rust,ignore
let primary = parse_current_source(source_path)?;
let view = project_translation_entries(&primary);
write_output(translation_path, vn_text::write_template(&view)?)?;

let current = parse_current_source(source_path)?;
let expected = project_translation_entries(&current);
let supplied = vn_text::read_template(&std::fs::read(translation_path)?)?;
vn_text::validate_shape(&expected, &supplied)?;
let localized = apply_translation_fields(current, supplied)?;
inject_and_rebuild(localized, output_path)?;
```

`project_translation_entries`, `apply_translation_fields`, and the final
injection are adapter code: they preserve all native metadata and enforce the
game's rules for writable names, controls, and encoding. Register these real
export/import operations with `vn-cli`; connect disk and font modules when the
game needs those stages. This yields the project flow:

`unpack → primary parse/extraction → secondary translation JSON → validated import → inject/rebuild → verify`

The secondary JSON has no file or entry IDs and preserves source order. Bind
each translation path to its exact source in the adapter's workspace mapping,
and verify the source before import. `validate_shape` cannot detect a same-size
wrong file or reordered entries.

## Adapter flow

1. Parse the source with the game's own parser and retain its complete structured
   representation and immutable metadata.
2. Convert the translation fields to `Entry` values in their original order and
   serialize them with `write_template`.
3. When importing edits, read them with `read_template` and call `validate_shape`
   against a fresh projection of the current source.
4. After validation, the adapter applies permitted `name` and `message` values
   to its own parsed records. It keeps all primary-extraction metadata and
   performs format-specific control, encoding, injection, and rebuilding.

`validate_shape` requires the same entry count and the same presence or absence
of `name` at every position. It does not compare text or name values because
those are translation fields. JSON entry order is the source order; this view
does not carry file or entry identifiers, so adapters must bind each JSON file
to its source and order. File discovery, paths, partial-file policy, display
normalization, metadata restoration, and applying edits remain adapter
responsibilities.
