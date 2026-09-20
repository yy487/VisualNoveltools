use crate::font::{self, EncodingPlan, EncodingPlanEntry};
use crate::{build_dat, g1_data, parse, replace_fat_file, sha256_hex, split_dat_resources, Result};
use encoding_rs::EUC_JP;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};

const WORKSPACE_FORMAT: &str = "genji-localization-workspace-v1";
const DOCUMENT_FORMAT: &str = "genji-indexed-text-v1";
const REBUILD_FORMAT: &str = "genji-localization-rebuild-v1";
const TEXT_TABLES: [usize; 6] = [20, 21, 22, 23, 24, 25];
const SCRIPT_TABLES: [(usize, usize); 7] = [
    (10, 20),
    (11, 20),
    (12, 21),
    (13, 22),
    (14, 23),
    (15, 24),
    (16, 25),
];

#[derive(Debug, Clone)]
pub struct LocalizationExtractReport {
    pub documents: usize,
    pub entries: usize,
    pub names: usize,
    pub choices: usize,
    pub references: usize,
    pub preserved_translations: usize,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LocalizationInjectReport {
    pub entries: usize,
    pub changed_entries: usize,
    pub changed_tables: usize,
    pub redrawn_slots: usize,
    pub g1_size: usize,
    pub g1_clusters: usize,
    pub output_fdi: PathBuf,
    pub output_font: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceManifest {
    #[serde(rename = "_format")]
    format: String,
    tool_version: String,
    source_file: String,
    source_sha256: String,
    documents: Vec<DocumentReference>,
    profile: String,
    summary: WorkspaceSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentReference {
    resource: usize,
    json_file: String,
    source_sha256: String,
    entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceSummary {
    documents: usize,
    entries: usize,
    names: usize,
    choices: usize,
    references: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextDocument {
    #[serde(rename = "_format")]
    format: String,
    #[serde(rename = "_archive")]
    archive: String,
    #[serde(rename = "_resource")]
    resource: usize,
    #[serde(rename = "_source_sha256")]
    source_sha256: String,
    #[serde(rename = "_encoding")]
    encoding: String,
    #[serde(rename = "_explicit_newline")]
    explicit_newline: String,
    #[serde(rename = "_automatic_wrap")]
    automatic_wrap: bool,
    entries: Vec<TextEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextEntry {
    #[serde(rename = "_index")]
    index: usize,
    #[serde(rename = "_offset")]
    offset: usize,
    #[serde(rename = "_size")]
    size: usize,
    #[serde(rename = "_source_bytes_hex")]
    source_bytes_hex: String,
    #[serde(rename = "_roles")]
    roles: Vec<String>,
    #[serde(
        rename = "_shared_pointer_ids",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    shared_pointer_ids: Vec<usize>,
    #[serde(rename = "_suffix_of", skip_serializing_if = "Vec::is_empty", default)]
    suffix_of: Vec<SuffixLink>,
    #[serde(rename = "_references", skip_serializing_if = "Vec::is_empty", default)]
    references: Vec<TextReference>,
    #[serde(rename = "_speakers", skip_serializing_if = "Vec::is_empty", default)]
    speakers: Vec<SpeakerContext>,
    #[serde(rename = "_scr_name", skip_serializing_if = "Option::is_none")]
    scr_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scr_msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SuffixLink {
    parent_id: usize,
    byte_delta: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct SpeakerContext {
    id: usize,
    source_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TextReference {
    script_resource: usize,
    instruction_offset: usize,
    window: u8,
    use_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    speaker_id: Option<usize>,
}

#[derive(Debug, Clone)]
struct RawText {
    id: usize,
    offset: usize,
    end: usize,
    bytes: Vec<u8>,
    text: String,
}

#[derive(Debug, Clone, Copy)]
struct DisplayCall {
    offset: usize,
    window: u8,
    text_id: usize,
}

#[derive(Debug, Clone)]
enum TextAtom {
    Character(char),
    Control(u16),
}

#[derive(Debug, Clone, Serialize)]
struct RebuildManifest {
    #[serde(rename = "_format")]
    format: String,
    source_file: String,
    source_sha256: String,
    output_fdi: String,
    output_fdi_sha256: String,
    font: RebuiltFont,
    tables: Vec<RebuiltTable>,
    summary: RebuildSummary,
}

#[derive(Debug, Clone, Serialize)]
struct RebuiltFont {
    output_file: String,
    sha256: String,
    face: String,
    redrawn_slots: usize,
    mappings: Vec<EncodingPlanEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct RebuiltTable {
    resource: usize,
    source_size: usize,
    output_size: usize,
    changed_entries: usize,
}

#[derive(Debug, Clone, Serialize)]
struct RebuildSummary {
    entries: usize,
    changed_entries: usize,
    changed_tables: usize,
    g1_size: usize,
    g1_clusters: usize,
}

pub fn extract_localization(
    input: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<LocalizationExtractReport> {
    reject_output_containing(output, &[input])?;
    let source = fs::read(input).map_err(|error| format!("{}: {error}", input.display()))?;
    let manifest = parse(&source)?;
    let resources = split_dat_resources(g1_data(&manifest)?)?;
    let memory = if overwrite && output.join("workspace.json").is_file() {
        read_translation_memory(output)?
    } else {
        HashMap::new()
    };
    validate_managed_directory(output, overwrite, WORKSPACE_FORMAT, "workspace.json")?;
    let staging = staging_sibling(output)?;
    fs::create_dir_all(staging.join("translation_json")).map_err(|e| e.to_string())?;
    fs::create_dir_all(staging.join("profile")).map_err(|e| e.to_string())?;

    let mut documents = Vec::new();
    let mut total_entries = 0usize;
    let mut total_names = 0usize;
    let mut total_choices = 0usize;
    let mut total_references = 0usize;
    let mut preserved = 0usize;
    for table in TEXT_TABLES {
        let mut document = make_document(table, &resources)?;
        for entry in &mut document.entries {
            let key = (table, entry.index, source_text(entry).to_owned());
            if let Some(value) = memory.get(&key) {
                set_editable_text(entry, value.clone())?;
                preserved += 1;
            }
        }
        total_entries += document.entries.len();
        total_names += document
            .entries
            .iter()
            .filter(|entry| entry.scr_name.is_some())
            .count();
        total_choices += document
            .entries
            .iter()
            .filter(|entry| entry.roles.iter().any(|role| role == "choice"))
            .count();
        total_references += document
            .entries
            .iter()
            .map(|entry| entry.references.len())
            .sum::<usize>();
        let json_file = format!("translation_json/G1_{table:04}.json");
        write_json(&staging.join(&json_file), &document)?;
        documents.push(DocumentReference {
            resource: table,
            json_file,
            source_sha256: document.source_sha256.clone(),
            entries: document.entries.len(),
        });
    }

    let source_file = input
        .file_name()
        .ok_or_else(|| "input FDI has no filename".to_string())?
        .to_string_lossy()
        .into_owned();
    let workspace = WorkspaceManifest {
        format: WORKSPACE_FORMAT.to_owned(),
        tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        source_file,
        source_sha256: sha256_hex(&source),
        documents,
        profile: "profile/project.json".to_owned(),
        summary: WorkspaceSummary {
            documents: TEXT_TABLES.len(),
            entries: total_entries,
            names: total_names,
            choices: total_choices,
            references: total_references,
        },
    };
    write_json(&staging.join("workspace.json"), &workspace)?;
    write_profile(&staging.join("profile/project.json"))?;
    commit_directory(&staging, output, overwrite)?;
    Ok(LocalizationExtractReport {
        documents: TEXT_TABLES.len(),
        entries: total_entries,
        names: total_names,
        choices: total_choices,
        references: total_references,
        preserved_translations: preserved,
        output: output.to_path_buf(),
    })
}

fn make_document(table: usize, resources: &[Vec<u8>]) -> Result<TextDocument> {
    let source = resources
        .get(table)
        .ok_or_else(|| format!("G1 resource {table:04} is missing"))?;
    let raw = parse_text_table(table, source)?;
    let text_by_id = raw
        .iter()
        .map(|entry| (entry.id, entry.text.clone()))
        .collect::<HashMap<_, _>>();
    let references = build_references(table, resources, &text_by_id);
    let name_ids = references
        .iter()
        .filter_map(|(id, refs)| {
            refs.iter()
                .any(|reference| reference.use_kind == "name")
                .then_some(*id)
        })
        .collect::<BTreeSet<_>>();

    let mut entries = Vec::with_capacity(raw.len());
    for item in &raw {
        let refs = references.get(&item.id).cloned().unwrap_or_default();
        let mut roles = refs
            .iter()
            .map(|reference| reference.use_kind.clone())
            .collect::<BTreeSet<_>>();
        if known_choice(table, item.id) {
            roles.insert("choice".to_owned());
        }
        if refs.is_empty() && roles.is_empty() {
            roles.insert("unreferenced".to_owned());
        }
        let speakers = refs
            .iter()
            .filter_map(|reference| reference.speaker_id)
            .filter_map(|id| {
                text_by_id.get(&id).map(|name| SpeakerContext {
                    id,
                    source_name: name.clone(),
                })
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let shared_pointer_ids = raw
            .iter()
            .filter(|other| other.id != item.id && other.offset == item.offset)
            .map(|other| other.id)
            .collect();
        let suffix_of = raw
            .iter()
            .filter(|parent| {
                parent.id != item.id && parent.offset < item.offset && item.offset < parent.end
            })
            .map(|parent| SuffixLink {
                parent_id: parent.id,
                byte_delta: item.offset - parent.offset,
            })
            .collect();
        let is_name = name_ids.contains(&item.id);
        entries.push(TextEntry {
            index: item.id,
            offset: item.offset,
            size: item.end - item.offset,
            source_bytes_hex: hex_encode(&item.bytes),
            roles: roles.into_iter().collect(),
            shared_pointer_ids,
            suffix_of,
            references: refs,
            speakers,
            scr_name: is_name.then(|| item.text.clone()),
            name: is_name.then(|| item.text.clone()),
            scr_msg: (!is_name).then(|| item.text.clone()),
            message: (!is_name).then(|| item.text.clone()),
        });
    }
    Ok(TextDocument {
        format: DOCUMENT_FORMAT.to_owned(),
        archive: "G1.DAT".to_owned(),
        resource: table,
        source_sha256: sha256_hex(source),
        encoding: "little-endian JIS X 0208 carrier codes; 0000 terminator".to_owned(),
        explicit_newline: "JSON \\n maps to text control 0x0010".to_owned(),
        automatic_wrap: true,
        entries,
    })
}

fn parse_text_table(table: usize, source: &[u8]) -> Result<Vec<RawText>> {
    if source.len() < 512 || source.len() % 256 != 0 {
        return Err(format!(
            "G1 resource {table:04} is not a 256-byte-aligned indexed text table"
        ));
    }
    let mut entries = Vec::new();
    for id in 0..256 {
        let pointer = u16::from_le_bytes([source[id * 2], source[id * 2 + 1]]) as usize;
        if pointer == 0 {
            continue;
        }
        if pointer < 512 || pointer >= source.len() || pointer % 2 != 0 {
            return Err(format!(
                "G1 resource {table:04} text {id} has invalid pointer 0x{pointer:X}"
            ));
        }
        let mut cursor = pointer;
        let mut text = String::new();
        loop {
            let raw = source.get(cursor..cursor + 2).ok_or_else(|| {
                format!("G1 resource {table:04} text {id} is missing its terminator")
            })?;
            cursor += 2;
            let value = u16::from_le_bytes([raw[0], raw[1]]);
            match value {
                0 => break,
                1..=15 => text.push_str(&format!("{{COLOR:{value:02X}}}")),
                0x10 => text.push('\n'),
                0x100..=0x1FF => text.push_str(&format!("{{GLYPH:{:02X}}}", value & 0xFF)),
                0xFFFF => text.push_str("{SCRIPT}"),
                _ => {
                    let row = (value >> 8) as u8;
                    let cell = value as u8;
                    if !(0x21..=0x7E).contains(&row) || !(0x21..=0x7E).contains(&cell) {
                        return Err(format!(
                            "G1 resource {table:04} text {id} has unknown code 0x{value:04X} at 0x{:X}",
                            cursor - 2
                        ));
                    }
                    let euc = [row | 0x80, cell | 0x80];
                    let decoded = EUC_JP
                        .decode_without_bom_handling_and_without_replacement(&euc)
                        .ok_or_else(|| {
                            format!(
                                "G1 resource {table:04} text {id} has invalid JIS code 0x{value:04X}"
                            )
                        })?;
                    text.push_str(&decoded);
                }
            }
        }
        entries.push(RawText {
            id,
            offset: pointer,
            end: cursor,
            bytes: source[pointer..cursor].to_vec(),
            text,
        });
    }
    Ok(entries)
}

fn build_references(
    table: usize,
    resources: &[Vec<u8>],
    text_by_id: &HashMap<usize, String>,
) -> BTreeMap<usize, Vec<TextReference>> {
    let mut result = BTreeMap::<usize, Vec<TextReference>>::new();
    for (script_id, script_table) in SCRIPT_TABLES {
        if script_table != table {
            continue;
        }
        let Some(script) = resources.get(script_id) else {
            continue;
        };
        let calls = scan_display_calls(script);
        let by_offset = calls
            .iter()
            .map(|call| (call.offset, *call))
            .collect::<HashMap<_, _>>();
        let mut name_offsets = BTreeSet::new();
        for call in &calls {
            let Some(next) = by_offset.get(&(call.offset + 5)) else {
                continue;
            };
            let source = text_by_id
                .get(&call.text_id)
                .map(String::as_str)
                .unwrap_or("");
            if next.window == call.window
                && (20..80).contains(&call.text_id)
                && !source.contains('\n')
                && source.chars().count() <= 12
            {
                name_offsets.insert(call.offset);
            }
        }
        for call in &calls {
            let previous = call.offset.checked_sub(5).and_then(|offset| {
                if name_offsets.contains(&offset) {
                    by_offset.get(&offset).copied()
                } else {
                    None
                }
            });
            let (use_kind, speaker_id): (&str, Option<usize>) =
                if name_offsets.contains(&call.offset) {
                    ("name", None)
                } else if let Some(previous) = previous.filter(|item| item.window == call.window) {
                    ("dialogue", Some(previous.text_id))
                } else if known_choice(table, call.text_id)
                    || script.get(call.offset + 5..call.offset + 8)
                        == Some([0xB1, 0x00, call.window].as_slice())
                {
                    ("choice", None)
                } else {
                    ("text", None)
                };
            result.entry(call.text_id).or_default().push(TextReference {
                script_resource: script_id,
                instruction_offset: call.offset,
                window: call.window,
                use_kind: use_kind.to_owned(),
                speaker_id,
            });
        }
    }
    result
}

fn scan_display_calls(script: &[u8]) -> Vec<DisplayCall> {
    let mut calls = Vec::new();
    for offset in 0..script.len().saturating_sub(4) {
        if script[offset] == 0xB0 && script[offset + 1] == 0 && script[offset + 3] == 0 {
            calls.push(DisplayCall {
                offset,
                window: script[offset + 2],
                text_id: script[offset + 4] as usize,
            });
        }
    }
    calls
}

fn known_choice(table: usize, id: usize) -> bool {
    (table == 20 && (10..=13).contains(&id))
        || ((21..=25).contains(&table) && (1..=10).contains(&id))
}

pub fn inject_localization(
    source_fdi: &Path,
    workspace_dir: &Path,
    output_dir: &Path,
    overwrite: bool,
) -> Result<LocalizationInjectReport> {
    reject_output_containing(output_dir, &[source_fdi, workspace_dir])?;
    let source =
        fs::read(source_fdi).map_err(|error| format!("{}: {error}", source_fdi.display()))?;
    let source_manifest = parse(&source)?;
    let original_g1 = g1_data(&source_manifest)?;
    let mut resources = split_dat_resources(original_g1)?;
    let workspace: WorkspaceManifest = read_json(&workspace_dir.join("workspace.json"))?;
    if workspace.format != WORKSPACE_FORMAT || workspace.source_sha256 != sha256_hex(&source) {
        return Err("translation workspace does not belong to this source FDI".into());
    }
    if workspace.documents.len() != TEXT_TABLES.len() {
        return Err("translation workspace does not contain all six text tables".into());
    }

    let mut documents = BTreeMap::new();
    for reference in &workspace.documents {
        if !TEXT_TABLES.contains(&reference.resource) {
            return Err(format!(
                "workspace contains unexpected text resource {}",
                reference.resource
            ));
        }
        let path = workspace_dir.join(safe_relative_path(&reference.json_file)?);
        let document: TextDocument = read_json(&path)?;
        if document.resource != reference.resource
            || document.source_sha256 != reference.source_sha256
            || document.entries.len() != reference.entries
            || documents.insert(reference.resource, document).is_some()
        {
            return Err(format!("{} does not match workspace.json", path.display()));
        }
    }

    let mut plan_strings = Vec::new();
    let mut total_entries = 0usize;
    for table in TEXT_TABLES {
        let expected = make_document(table, &resources)?;
        let document = documents
            .get(&table)
            .ok_or_else(|| format!("workspace is missing G1_{table:04}.json"))?;
        validate_document(document, &expected)?;
        total_entries += document.entries.len();
        for entry in &document.entries {
            plan_strings.push(display_characters(editable_text(entry)?)?);
        }
    }
    let plan = EncodingPlan::build(plan_strings.iter().map(String::as_str))?;
    let font_build = font::prepare_font(&plan.requests(), &BTreeSet::new())?;

    let mut table_reports = Vec::new();
    let mut changed_entries = 0usize;
    for table in TEXT_TABLES {
        let document = &documents[&table];
        let source_table = &resources[table];
        let (rebuilt, changed) = rebuild_text_resource(source_table, document, &plan)?;
        table_reports.push(RebuiltTable {
            resource: table,
            source_size: source_table.len(),
            output_size: rebuilt.len(),
            changed_entries: changed,
        });
        changed_entries += changed;
        resources[table] = rebuilt;
    }
    let changed_tables = table_reports
        .iter()
        .filter(|table| table.changed_entries != 0)
        .count();
    let rebuilt_g1 = build_dat(&resources)?;
    let rebuilt_fdi = if changed_entries == 0 {
        source.clone()
    } else {
        replace_fat_file(&source, "G1.DAT", &rebuilt_g1)?
    };

    validate_managed_directory(
        output_dir,
        overwrite,
        REBUILD_FORMAT,
        "rebuild_manifest.json",
    )?;
    let staging = staging_sibling(output_dir)?;
    fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    let output_name = format!(
        "{}_translated.FDI",
        source_fdi.file_stem().unwrap_or_default().to_string_lossy()
    );
    fs::write(staging.join(&output_name), &rebuilt_fdi).map_err(|e| e.to_string())?;
    fs::write(staging.join("font.tmp"), &font_build.bytes).map_err(|e| e.to_string())?;
    let rebuild = RebuildManifest {
        format: REBUILD_FORMAT.to_owned(),
        source_file: workspace.source_file,
        source_sha256: sha256_hex(&source),
        output_fdi: output_name.clone(),
        output_fdi_sha256: sha256_hex(&rebuilt_fdi),
        font: RebuiltFont {
            output_file: "font.tmp".to_owned(),
            sha256: sha256_hex(&font_build.bytes),
            face: font::FONT_FACE.to_owned(),
            redrawn_slots: font_build.patched_glyphs,
            mappings: plan.manifest_entries()?,
        },
        tables: table_reports,
        summary: RebuildSummary {
            entries: total_entries,
            changed_entries,
            changed_tables,
            g1_size: rebuilt_g1.len(),
            g1_clusters: rebuilt_g1.len().div_ceil(1024),
        },
    };
    write_json(&staging.join("rebuild_manifest.json"), &rebuild)?;
    commit_directory(&staging, output_dir, overwrite)?;
    Ok(LocalizationInjectReport {
        entries: total_entries,
        changed_entries,
        changed_tables,
        redrawn_slots: font_build.patched_glyphs,
        g1_size: rebuilt_g1.len(),
        g1_clusters: rebuilt_g1.len().div_ceil(1024),
        output_fdi: output_dir.join(output_name),
        output_font: output_dir.join("font.tmp"),
    })
}

fn validate_document(document: &TextDocument, expected: &TextDocument) -> Result<()> {
    if document.format != expected.format
        || document.archive != expected.archive
        || document.resource != expected.resource
        || document.source_sha256 != expected.source_sha256
        || document.encoding != expected.encoding
        || document.explicit_newline != expected.explicit_newline
        || document.automatic_wrap != expected.automatic_wrap
        || document.entries.len() != expected.entries.len()
    {
        return Err(format!(
            "G1_{:04}.json immutable document metadata was modified",
            expected.resource
        ));
    }
    for (actual, source) in document.entries.iter().zip(&expected.entries) {
        if actual.index != source.index
            || actual.offset != source.offset
            || actual.size != source.size
            || actual.source_bytes_hex != source.source_bytes_hex
            || actual.roles != source.roles
            || actual.shared_pointer_ids != source.shared_pointer_ids
            || actual.suffix_of != source.suffix_of
            || actual.references != source.references
            || actual.speakers != source.speakers
            || actual.scr_name != source.scr_name
            || actual.scr_msg != source.scr_msg
            || actual.name.is_some() != source.name.is_some()
            || actual.message.is_some() != source.message.is_some()
        {
            return Err(format!(
                "G1_{:04}.json entry {}: source or underscore fields were modified",
                expected.resource, source.index
            ));
        }
    }
    Ok(())
}

fn rebuild_text_resource(
    source: &[u8],
    document: &TextDocument,
    plan: &EncodingPlan,
) -> Result<(Vec<u8>, usize)> {
    let changed = document
        .entries
        .iter()
        .filter(|entry| editable_text(entry).is_ok_and(|text| text != source_text(entry)))
        .count();
    if changed == 0 {
        return Ok((source.to_vec(), 0));
    }
    let mut by_id = BTreeMap::new();
    for entry in &document.entries {
        by_id.insert(entry.index, entry);
    }
    let mut output = vec![0u8; 512];
    for id in 0..256 {
        let original_pointer = u16::from_le_bytes([source[id * 2], source[id * 2 + 1]]);
        if original_pointer == 0 {
            continue;
        }
        let entry = by_id
            .get(&id)
            .ok_or_else(|| format!("text resource {} is missing entry {id}", document.resource))?;
        if output.len() > u16::MAX as usize {
            return Err(format!(
                "text resource {} exceeds its 16-bit pointer range",
                document.resource
            ));
        }
        let pointer = output.len() as u16;
        output[id * 2..id * 2 + 2].copy_from_slice(&pointer.to_le_bytes());
        output.extend_from_slice(&encode_text(editable_text(entry)?, plan)?);
    }
    let aligned = output.len().div_ceil(256) * 256;
    if aligned / 256 > u8::MAX as usize {
        return Err(format!(
            "text resource {} needs {} DAT blocks; the limit is 255",
            document.resource,
            aligned / 256
        ));
    }
    output.resize(aligned, 0);
    Ok((output, changed))
}

fn encode_text(text: &str, plan: &EncodingPlan) -> Result<Vec<u8>> {
    let atoms = parse_editable_text(text)?;
    let mut output = Vec::with_capacity(atoms.len() * 2 + 2);
    for atom in atoms {
        match atom {
            TextAtom::Control(value) => output.extend_from_slice(&value.to_le_bytes()),
            TextAtom::Character(character) => {
                let carrier = plan.carrier_for(character)?;
                let cp932 = font::cp932_for_carrier(carrier)?;
                let jis = font::cp932_to_jis(cp932)?;
                output.extend_from_slice(&[jis[1], jis[0]]);
            }
        }
    }
    output.extend_from_slice(&[0, 0]);
    Ok(output)
}

fn display_characters(text: &str) -> Result<String> {
    Ok(parse_editable_text(text)?
        .into_iter()
        .filter_map(|atom| match atom {
            TextAtom::Character(character) => Some(character),
            TextAtom::Control(_) => None,
        })
        .collect())
}

fn parse_editable_text(text: &str) -> Result<Vec<TextAtom>> {
    let mut atoms = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((offset, character)) = chars.next() {
        match character {
            '\n' => atoms.push(TextAtom::Control(0x10)),
            '\r' | '\0' => {
                return Err(format!(
                    "text contains forbidden control U+{:04X}",
                    character as u32
                ));
            }
            '{' => {
                let end = text[offset..]
                    .find('}')
                    .map(|relative| offset + relative)
                    .ok_or_else(|| format!("unclosed control token at character byte {offset}"))?;
                while chars.peek().is_some_and(|(position, _)| *position <= end) {
                    chars.next();
                }
                let token = &text[offset + 1..end];
                let value = if token == "SCRIPT" {
                    0xFFFF
                } else if let Some(hex) = token.strip_prefix("COLOR:") {
                    let value = parse_hex_byte(hex, token)? as u16;
                    if !(1..=15).contains(&value) {
                        return Err(format!("invalid color control {{{token}}}"));
                    }
                    value
                } else if let Some(hex) = token.strip_prefix("GLYPH:") {
                    0x100 | parse_hex_byte(hex, token)? as u16
                } else {
                    return Err(format!(
                        "unknown control {{{token}}}; use fullwidth braces for literal text"
                    ));
                };
                atoms.push(TextAtom::Control(value));
            }
            '}' => {
                return Err("unmatched }; use the fullwidth character ｝ for literal text".into());
            }
            _ => atoms.push(TextAtom::Character(font::normalize_character(character)?)),
        }
    }
    Ok(atoms)
}

fn parse_hex_byte(hex: &str, token: &str) -> Result<u8> {
    if hex.len() != 2 {
        return Err(format!(
            "control {{{token}}} requires exactly two hex digits"
        ));
    }
    u8::from_str_radix(hex, 16).map_err(|_| format!("invalid control {{{token}}}"))
}

fn source_text(entry: &TextEntry) -> &str {
    entry
        .scr_name
        .as_deref()
        .or(entry.scr_msg.as_deref())
        .expect("text entry always has a source field")
}

fn editable_text(entry: &TextEntry) -> Result<&str> {
    entry
        .name
        .as_deref()
        .or(entry.message.as_deref())
        .ok_or_else(|| format!("text entry {} has no editable field", entry.index))
}

fn set_editable_text(entry: &mut TextEntry, value: String) -> Result<()> {
    if entry.name.is_some() {
        entry.name = Some(value);
    } else if entry.message.is_some() {
        entry.message = Some(value);
    } else {
        return Err(format!("text entry {} has no editable field", entry.index));
    }
    Ok(())
}

fn read_translation_memory(root: &Path) -> Result<HashMap<(usize, usize, String), String>> {
    let workspace: WorkspaceManifest = read_json(&root.join("workspace.json"))?;
    if workspace.format != WORKSPACE_FORMAT {
        return Err("existing output is not a Genji localization workspace".into());
    }
    let mut memory = HashMap::new();
    for reference in workspace.documents {
        let document: TextDocument =
            read_json(&root.join(safe_relative_path(&reference.json_file)?))?;
        for entry in document.entries {
            let key = (
                document.resource,
                entry.index,
                source_text(&entry).to_owned(),
            );
            memory.insert(key, editable_text(&entry)?.to_owned());
        }
    }
    Ok(memory)
}

fn safe_relative_path(value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe relative path in workspace: {value}"));
    }
    Ok(path.to_path_buf())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn write_profile(path: &Path) -> Result<()> {
    let profile = serde_json::json!({
        "_format": "genji-project-profile-v1",
        "editable_fields": {
            "name": "speaker-name slots; immutable source is _scr_name",
            "message": "dialogue, narration, choices, and other text; immutable source is scr_msg"
        },
        "immutable_fields": "scr_msg, _scr_name, and every underscore-prefixed field",
        "name_message_pairing": "consecutive immediate B0 calls to the same window; reusable messages list all observed speakers in _speakers",
        "encoding": "little-endian JIS X 0208 carrier codes with a 0000 terminator",
        "controls": {
            "JSON newline": "0x0010 explicit display newline",
            "{COLOR:01}..{COLOR:0F}": "text color/mask controls 0x0001..0x000F",
            "{GLYPH:00}..{GLYPH:FF}": "custom glyph controls 0x0100..0x01FF",
            "{SCRIPT}": "0xFFFF interpreter call"
        },
        "automatic_wrap": true,
        "choices": "choice is a usage role; options use ordinary B0 text and script B1/menu logic",
        "font": {
            "output": "font.tmp beside the rebuilt FDI",
            "face": font::FONT_FACE,
            "mapping": "deterministic CP932/JIS carrier slots from subs_cn_jp.json"
        },
        "repack": "rebuild changed text tables, G1.DAT, the G1 FAT12 chain, both FAT copies, and the root file size"
    });
    write_json(path, &profile)
}

fn validate_managed_directory(
    output: &Path,
    overwrite: bool,
    expected_format: &str,
    marker: &str,
) -> Result<()> {
    if !output.exists() {
        return Ok(());
    }
    if !output.is_dir() {
        return Err(format!(
            "output exists and is not a directory: {}",
            output.display()
        ));
    }
    if !overwrite {
        return Err(format!(
            "output exists: {} (use --overwrite)",
            output.display()
        ));
    }
    if fs::read_dir(output)
        .map_err(|e| e.to_string())?
        .next()
        .is_none()
    {
        return Ok(());
    }
    let marker_path = output.join(marker);
    let value: serde_json::Value = read_json(&marker_path).map_err(|_| {
        format!(
            "refusing to replace non-managed directory: {}",
            output.display()
        )
    })?;
    if value.get("_format").and_then(|value| value.as_str()) != Some(expected_format) {
        return Err(format!(
            "refusing to replace directory with a different format: {}",
            output.display()
        ));
    }
    Ok(())
}

fn reject_output_containing(output: &Path, inputs: &[&Path]) -> Result<()> {
    let output_absolute = absolute_path(output)?;
    for input in inputs {
        let input_absolute = absolute_path(input)?;
        if input_absolute == output_absolute || input_absolute.starts_with(&output_absolute) {
            return Err(format!(
                "output {} would contain or replace input {}",
                output.display(),
                input.display()
            ));
        }
    }
    Ok(())
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()));
    }
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .map_err(|e| e.to_string())
    }
}

fn staging_sibling(output: &Path) -> Result<PathBuf> {
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    let name = output
        .file_name()
        .ok_or_else(|| format!("output has no directory name: {}", output.display()))?
        .to_string_lossy();
    let staging = parent.join(format!(".{name}.staging-{}", std::process::id()));
    if staging.exists() {
        return Err(format!(
            "staging path already exists: {}",
            staging.display()
        ));
    }
    Ok(staging)
}

fn commit_directory(staging: &Path, output: &Path, overwrite: bool) -> Result<()> {
    if output.exists() {
        if !overwrite {
            return Err(format!("output exists: {}", output.display()));
        }
        fs::remove_dir_all(output).map_err(|error| format!("{}: {error}", output.display()))?;
    }
    fs::rename(staging, output).map_err(|error| format!("{}: {error}", output.display()))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0F) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_editable_controls() {
        let atoms = parse_editable_text("甲\n乙{COLOR:03}{GLYPH:7F}{SCRIPT}").unwrap();
        assert_eq!(atoms.len(), 6);
        assert!(matches!(atoms[1], TextAtom::Control(0x10)));
        assert!(matches!(atoms[3], TextAtom::Control(3)));
        assert!(matches!(atoms[4], TextAtom::Control(0x17F)));
        assert!(matches!(atoms[5], TextAtom::Control(0xFFFF)));
    }
}
