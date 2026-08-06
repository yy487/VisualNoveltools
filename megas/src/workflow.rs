use crate::font::{build_font_pair_for_targets, missing_ttf_glyphs};
use crate::glyph::{GlyphDictionary, TranslationGlyphUseKind};
use crate::inject::{inject_path, InjectReport};
use crate::text_json::{read_entries, write_entries, TextEntry};
use crate::workspace::{
    load_workspace_translation_entries, resolve_translation_workspace, TranslationWorkspacePaths,
};
use crate::{ToolError, ToolResult};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TranslationBuildReport {
    pub json_input: PathBuf,
    pub script_root: PathBuf,
    pub output_dirs: Vec<PathBuf>,
    pub json_entries: usize,
    pub patched: usize,
    pub unchanged: usize,
    pub required_glyphs: usize,
    pub font_pairs: usize,
    pub rendered_slots: usize,
    pub donor_fonts_used: usize,
}

#[derive(Debug, Clone)]
pub struct TranslationBuildPlan {
    pub json_input: PathBuf,
    pub script_root: PathBuf,
    pub output_dirs: Vec<PathBuf>,
    pub json_entries: usize,
    pub script_files: usize,
    pub required_glyphs: usize,
    pub font_pairs: usize,
}

#[derive(Debug, Clone)]
pub struct TranslationFontBuildPlan {
    pub json_input: PathBuf,
    pub source_root: PathBuf,
    pub output_dirs: Vec<PathBuf>,
    pub json_entries: usize,
    pub required_glyphs: usize,
    pub font_pairs: usize,
}

#[derive(Debug, Clone)]
pub struct TranslationFontBuildReport {
    pub json_input: PathBuf,
    pub output_dirs: Vec<PathBuf>,
    pub required_glyphs: usize,
    pub font_pairs: usize,
    pub rendered_slots: usize,
    pub donor_fonts_used: usize,
}

#[derive(Debug, Clone)]
pub enum TranslationBuildProgress {
    InjectingScripts,
    RenderingFont {
        current: usize,
        total: usize,
        input_png: PathBuf,
    },
    Finalizing,
}

#[derive(Debug, Clone)]
struct FontPair {
    bin: PathBuf,
    png: PathBuf,
}

#[derive(Debug, Clone)]
struct SourceLayout {
    script_root: PathBuf,
    font_pairs: Vec<FontPair>,
    unified_output: bool,
    workspace_root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct TranslationContext {
    entries: Vec<TextEntry>,
    json_input: PathBuf,
    workspace: Option<TranslationWorkspacePaths>,
}

pub fn load_translation_entries(
    path: &Path,
    dictionary: &GlyphDictionary,
) -> ToolResult<Vec<TextEntry>> {
    Ok(load_translation_context(path, dictionary)?.entries)
}

fn load_translation_context(
    path: &Path,
    dictionary: &GlyphDictionary,
) -> ToolResult<TranslationContext> {
    if let Some(workspace) = resolve_translation_workspace(path)? {
        let entries = load_workspace_translation_entries(&workspace, dictionary)?;
        return Ok(TranslationContext {
            entries,
            json_input: workspace.translation_root.clone(),
            workspace: Some(workspace),
        });
    }
    Ok(TranslationContext {
        entries: load_regular_translation_entries(path)?,
        json_input: path.to_path_buf(),
        workspace: None,
    })
}

fn load_regular_translation_entries(path: &Path) -> ToolResult<Vec<TextEntry>> {
    if path.is_file() {
        return read_entries(path);
    }
    if !path.is_dir() {
        return Err(ToolError(format!(
            "translation JSON input '{}' is not a file or directory",
            path.display()
        )));
    }
    let mut files = Vec::new();
    collect_json_files(path, &mut files)?;
    files.sort();
    let mut entries = Vec::new();
    for file in files {
        let bytes = fs::read(&file).map_err(|error| {
            ToolError(format!(
                "cannot read translation JSON '{}': {error}",
                file.display()
            ))
        })?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
            ToolError(format!(
                "cannot parse translation JSON '{}': {error}",
                file.display()
            ))
        })?;
        if !value.is_array() {
            continue;
        }
        let mut file_entries: Vec<TextEntry> = serde_json::from_value(value).map_err(|error| {
            ToolError(format!(
                "translation JSON '{}' has an invalid entry: {error}",
                file.display()
            ))
        })?;
        entries.append(&mut file_entries);
    }
    if entries.is_empty() {
        return Err(ToolError(format!(
            "translation JSON directory '{}' contains no translation entries",
            path.display()
        )));
    }
    Ok(entries)
}

pub fn required_render_targets(
    json_input: &Path,
    dictionary: &GlyphDictionary,
) -> ToolResult<BTreeSet<char>> {
    let entries = load_translation_context(json_input, dictionary)?.entries;
    let targets = scan_required_targets(&entries, dictionary)?;
    validate_slot_conflicts(&entries, dictionary)?;
    Ok(targets)
}

pub fn plan_translation_resources(
    json_input: &Path,
    dictionary: &GlyphDictionary,
) -> ToolResult<TranslationBuildPlan> {
    let context = load_translation_context(json_input, dictionary)?;
    let targets = scan_required_targets(&context.entries, dictionary)?;
    validate_slot_conflicts(&context.entries, dictionary)?;
    let layout = discover_context_layout(&context)?;
    let script_files = count_source_script_files(&layout.script_root)?;
    Ok(TranslationBuildPlan {
        json_input: context.json_input,
        script_root: layout.script_root.clone(),
        output_dirs: output_directories(&layout),
        json_entries: context.entries.len(),
        script_files,
        required_glyphs: targets.len(),
        font_pairs: layout.font_pairs.len(),
    })
}

pub fn plan_translation_fonts(
    json_input: &Path,
    dictionary: &GlyphDictionary,
) -> ToolResult<TranslationFontBuildPlan> {
    let context = load_translation_context(json_input, dictionary)?;
    let targets = scan_required_targets(&context.entries, dictionary)?;
    validate_slot_conflicts(&context.entries, dictionary)?;
    let layout = discover_context_layout(&context)?;
    Ok(TranslationFontBuildPlan {
        json_input: context.json_input,
        source_root: layout.script_root.clone(),
        output_dirs: font_only_output_directories(&layout),
        json_entries: context.entries.len(),
        required_glyphs: targets.len(),
        font_pairs: layout.font_pairs.len(),
    })
}

pub fn build_translation_resources(
    json_input: &Path,
    main_font: &Path,
    donors: &[PathBuf],
    overwrite: bool,
    dictionary: &GlyphDictionary,
) -> ToolResult<TranslationBuildReport> {
    build_translation_resources_impl(json_input, main_font, donors, overwrite, dictionary, None)
}

pub fn build_translation_resources_with_progress(
    json_input: &Path,
    main_font: &Path,
    donors: &[PathBuf],
    overwrite: bool,
    dictionary: &GlyphDictionary,
    progress: &mut dyn FnMut(TranslationBuildProgress),
) -> ToolResult<TranslationBuildReport> {
    build_translation_resources_impl(
        json_input,
        main_font,
        donors,
        overwrite,
        dictionary,
        Some(progress),
    )
}

fn build_translation_resources_impl(
    json_input: &Path,
    main_font: &Path,
    donors: &[PathBuf],
    overwrite: bool,
    dictionary: &GlyphDictionary,
    mut progress: Option<&mut dyn FnMut(TranslationBuildProgress)>,
) -> ToolResult<TranslationBuildReport> {
    if !main_font.is_file() {
        return Err(ToolError(format!(
            "main redraw font '{}' does not exist",
            main_font.display()
        )));
    }
    for donor in donors {
        if !donor.is_file() {
            return Err(ToolError(format!(
                "supplementary font '{}' does not exist",
                donor.display()
            )));
        }
    }
    let context = load_translation_context(json_input, dictionary)?;
    let targets = scan_required_targets(&context.entries, dictionary)?;
    validate_slot_conflicts(&context.entries, dictionary)?;
    let mut remaining = missing_ttf_glyphs(main_font, &targets)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    for donor in donors {
        let donor_missing = missing_ttf_glyphs(donor, &remaining)?
            .into_iter()
            .collect::<BTreeSet<_>>();
        remaining = donor_missing;
    }
    if !remaining.is_empty() {
        return Err(ToolError(format_missing_ttf(&remaining)));
    }

    let layout = discover_context_layout(&context)?;
    let output_dirs = output_directories(&layout);
    for output in &output_dirs {
        if output.exists() && !overwrite {
            return Err(ToolError(format!(
                "output directory already exists: '{}'",
                output.display()
            )));
        }
    }
    for output in &output_dirs {
        if output.exists() {
            fs::remove_dir_all(output).map_err(|error| {
                ToolError(format!(
                    "cannot remove existing output '{}': {error}",
                    output.display()
                ))
            })?;
        }
    }

    let operation = (|| -> ToolResult<(InjectReport, usize, usize)> {
        let script_output = &output_dirs[0];
        let stage_script_output = script_output
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!(".mpk-tool-chs-stage-{}", std::process::id()));
        if stage_script_output.exists() {
            return Err(ToolError(format!(
                "temporary chs staging path already exists: '{}'",
                stage_script_output.display()
            )));
        }
        if let Some(callback) = progress.as_deref_mut() {
            callback(TranslationBuildProgress::InjectingScripts);
        }
        let inject = inject_translation_json(
            &layout.script_root,
            &context.json_input,
            &stage_script_output,
            &context.entries,
            dictionary,
        )?;
        let target_set = targets.iter().copied().collect::<HashSet<_>>();
        let mut rendered_slots = 0;
        let mut donor_fonts_used = 0;
        for (pair_index, pair) in layout.font_pairs.iter().enumerate() {
            if let Some(callback) = progress.as_deref_mut() {
                callback(TranslationBuildProgress::RenderingFont {
                    current: pair_index + 1,
                    total: layout.font_pairs.len(),
                    input_png: pair.png.clone(),
                });
            }
            let (output_bin, output_png) = if layout.unified_output {
                let bin_relative = pair
                    .bin
                    .strip_prefix(&layout.script_root)
                    .map_err(|error| {
                        ToolError(format!(
                            "font '{}' is outside script root '{}': {error}",
                            pair.bin.display(),
                            layout.script_root.display()
                        ))
                    })?;
                let png_relative = pair
                    .png
                    .strip_prefix(&layout.script_root)
                    .map_err(|error| {
                        ToolError(format!(
                            "font '{}' is outside script root '{}': {error}",
                            pair.png.display(),
                            layout.script_root.display()
                        ))
                    })?;
                (
                    stage_script_output.join(bin_relative),
                    stage_script_output.join(png_relative),
                )
            } else {
                let output_dir = pair
                    .bin
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join("chs");
                (
                    output_dir.join(pair.bin.file_name().ok_or_else(|| {
                        ToolError(format!("invalid font BIN path '{}'", pair.bin.display()))
                    })?),
                    output_dir.join(pair.png.file_name().ok_or_else(|| {
                        ToolError(format!("invalid font PNG path '{}'", pair.png.display()))
                    })?),
                )
            };
            if let Some(parent) = output_bin.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    ToolError(format!(
                        "cannot create font output directory '{}': {error}",
                        parent.display()
                    ))
                })?;
            }
            let report = build_font_pair_replacing(
                &pair.bin,
                &pair.png,
                main_font,
                donors,
                &output_bin,
                &output_png,
                &target_set,
            )?;
            rendered_slots += report.rendered_slots;
            donor_fonts_used = donor_fonts_used.max(report.donor_fonts_used);
        }
        if let Some(callback) = progress.as_deref_mut() {
            callback(TranslationBuildProgress::Finalizing);
        }
        fs::rename(&stage_script_output, script_output).map_err(|error| {
            ToolError(format!(
                "cannot finalize script chs directory '{}': {error}",
                script_output.display()
            ))
        })?;
        Ok((inject, rendered_slots, donor_fonts_used))
    })();
    match operation {
        Ok((inject, rendered_slots, donor_fonts_used)) => Ok(TranslationBuildReport {
            json_input: context.json_input,
            script_root: layout.script_root,
            output_dirs,
            json_entries: inject.json_entries,
            patched: inject.patched,
            unchanged: inject.unchanged,
            required_glyphs: targets.len(),
            font_pairs: layout.font_pairs.len(),
            rendered_slots,
            donor_fonts_used,
        }),
        Err(error) => {
            let stage_script_output = output_dirs[0]
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(format!(".mpk-tool-chs-stage-{}", std::process::id()));
            let _ = fs::remove_dir_all(&stage_script_output);
            for output in &output_dirs {
                let _ = fs::remove_dir_all(output);
            }
            Err(error)
        }
    }
}

pub fn build_translation_fonts_with_progress(
    json_input: &Path,
    main_font: &Path,
    donors: &[PathBuf],
    overwrite: bool,
    dictionary: &GlyphDictionary,
    progress: &mut dyn FnMut(TranslationBuildProgress),
) -> ToolResult<TranslationFontBuildReport> {
    if !main_font.is_file() {
        return Err(ToolError(format!(
            "main redraw font '{}' does not exist",
            main_font.display()
        )));
    }
    for donor in donors {
        if !donor.is_file() {
            return Err(ToolError(format!(
                "supplementary font '{}' does not exist",
                donor.display()
            )));
        }
    }
    let context = load_translation_context(json_input, dictionary)?;
    let targets = scan_required_targets(&context.entries, dictionary)?;
    validate_slot_conflicts(&context.entries, dictionary)?;
    let mut remaining = missing_ttf_glyphs(main_font, &targets)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    for donor in donors {
        remaining = missing_ttf_glyphs(donor, &remaining)?.into_iter().collect();
    }
    if !remaining.is_empty() {
        return Err(ToolError(format_missing_ttf(&remaining)));
    }
    let layout = discover_context_layout(&context)?;
    let output_dirs = font_only_output_directories(&layout);
    for output in &output_dirs {
        if output.exists() && !overwrite {
            return Err(ToolError(format!(
                "font output directory already exists: '{}'",
                output.display()
            )));
        }
    }
    for output in &output_dirs {
        if output.exists() {
            fs::remove_dir_all(output).map_err(|error| {
                ToolError(format!(
                    "cannot remove existing font output '{}': {error}",
                    output.display()
                ))
            })?;
        }
        fs::create_dir_all(output).map_err(|error| {
            ToolError(format!(
                "cannot create font output directory '{}': {error}",
                output.display()
            ))
        })?;
    }

    let operation = (|| -> ToolResult<(usize, usize)> {
        let target_set = targets.iter().copied().collect::<HashSet<_>>();
        let mut rendered_slots = 0;
        let mut donor_fonts_used = 0;
        for (pair_index, pair) in layout.font_pairs.iter().enumerate() {
            progress(TranslationBuildProgress::RenderingFont {
                current: pair_index + 1,
                total: layout.font_pairs.len(),
                input_png: pair.png.clone(),
            });
            let (output_bin, output_png) = font_only_pair_outputs(&layout, pair)?;
            if let Some(parent) = output_bin.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    ToolError(format!(
                        "cannot create font output directory '{}': {error}",
                        parent.display()
                    ))
                })?;
            }
            let report = build_font_pair_for_targets(
                &pair.bin,
                &pair.png,
                main_font,
                donors,
                &output_bin,
                &output_png,
                None,
                &target_set,
            )?;
            rendered_slots += report.rendered_slots;
            donor_fonts_used = donor_fonts_used.max(report.donor_fonts_used);
        }
        progress(TranslationBuildProgress::Finalizing);
        Ok((rendered_slots, donor_fonts_used))
    })();
    match operation {
        Ok((rendered_slots, donor_fonts_used)) => Ok(TranslationFontBuildReport {
            json_input: context.json_input,
            output_dirs,
            required_glyphs: targets.len(),
            font_pairs: layout.font_pairs.len(),
            rendered_slots,
            donor_fonts_used,
        }),
        Err(error) => {
            for output in &output_dirs {
                let _ = fs::remove_dir_all(output);
            }
            Err(error)
        }
    }
}

fn output_directories(layout: &SourceLayout) -> Vec<PathBuf> {
    let mut output_dirs = vec![layout
        .workspace_root
        .as_ref()
        .map(|root| root.join("chs"))
        .unwrap_or_else(|| layout.script_root.join("chs"))];
    if !layout.unified_output {
        for pair in &layout.font_pairs {
            output_dirs.push(
                pair.bin
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join("chs"),
            );
        }
    }
    dedup_paths(&mut output_dirs);
    output_dirs
}

fn font_only_output_directories(layout: &SourceLayout) -> Vec<PathBuf> {
    if let Some(root) = &layout.workspace_root {
        return vec![root.join("font_chs")];
    }
    let mut output_dirs = layout
        .font_pairs
        .iter()
        .map(|pair| {
            pair.bin
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("chs")
        })
        .collect::<Vec<_>>();
    dedup_paths(&mut output_dirs);
    output_dirs
}

fn font_only_pair_outputs(
    layout: &SourceLayout,
    pair: &FontPair,
) -> ToolResult<(PathBuf, PathBuf)> {
    if let Some(root) = &layout.workspace_root {
        let bin_relative = pair
            .bin
            .strip_prefix(&layout.script_root)
            .map_err(|error| {
                ToolError(format!(
                    "font '{}' is outside workspace source '{}': {error}",
                    pair.bin.display(),
                    layout.script_root.display()
                ))
            })?;
        let png_relative = pair
            .png
            .strip_prefix(&layout.script_root)
            .map_err(|error| {
                ToolError(format!(
                    "font '{}' is outside workspace source '{}': {error}",
                    pair.png.display(),
                    layout.script_root.display()
                ))
            })?;
        return Ok((
            root.join("font_chs").join(bin_relative),
            root.join("font_chs").join(png_relative),
        ));
    }
    let output_dir = pair
        .bin
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("chs");
    Ok((
        output_dir.join(
            pair.bin.file_name().ok_or_else(|| {
                ToolError(format!("invalid font BIN path '{}'", pair.bin.display()))
            })?,
        ),
        output_dir.join(
            pair.png.file_name().ok_or_else(|| {
                ToolError(format!("invalid font PNG path '{}'", pair.png.display()))
            })?,
        ),
    ))
}

fn scan_required_targets(
    entries: &[TextEntry],
    dictionary: &GlyphDictionary,
) -> ToolResult<BTreeSet<char>> {
    let mut targets = BTreeSet::new();
    let mut unmapped = BTreeSet::new();
    for entry in entries {
        for text in std::iter::once(&entry.message).chain(entry.name.as_ref()) {
            let scan = dictionary.scan_translation_text(text)?;
            targets.extend(scan.render_targets);
            unmapped.extend(scan.unmapped);
        }
    }
    if !unmapped.is_empty() {
        return Err(ToolError(format_missing_characters(&unmapped)));
    }
    Ok(targets)
}

#[derive(Debug, Default)]
struct SlotUsage {
    targets: BTreeSet<char>,
    literals: BTreeSet<char>,
    literal_locations: BTreeSet<String>,
    markup_locations: BTreeSet<String>,
}

fn validate_slot_conflicts(entries: &[TextEntry], dictionary: &GlyphDictionary) -> ToolResult<()> {
    let mut slots = BTreeMap::<u16, SlotUsage>::new();
    for entry in entries {
        for (field, text) in [("message", &entry.message)]
            .into_iter()
            .chain(entry.name.as_ref().map(|name| ("name", name)))
        {
            let location = format!("{}#{} {field}", entry.file, entry.index);
            for glyph_use in dictionary.scan_translation_glyph_uses(text)? {
                let slot = slots.entry(glyph_use.index).or_default();
                match glyph_use.kind {
                    TranslationGlyphUseKind::MappedTarget(character) => {
                        slot.targets.insert(character);
                    }
                    TranslationGlyphUseKind::Literal(character) => {
                        slot.literals.insert(character);
                        slot.literal_locations.insert(location.clone());
                    }
                    TranslationGlyphUseKind::Markup => {
                        slot.markup_locations.insert(location.clone());
                    }
                }
            }
        }
    }

    let conflicts = slots
        .into_iter()
        .filter(|(_, usage)| {
            !usage.targets.is_empty()
                && (usage.targets.len() > 1
                    || !usage.literals.is_empty()
                    || !usage.markup_locations.is_empty())
        })
        .collect::<Vec<_>>();
    if conflicts.is_empty() {
        return Ok(());
    }

    let mut lines = Vec::with_capacity(conflicts.len());
    for (index, usage) in conflicts {
        let mappings = usage
            .targets
            .iter()
            .map(|target| {
                let carrier = dictionary
                    .carrier_for(*target)
                    .expect("mapped translation target");
                format!("{target:?}->{carrier:?}")
            })
            .collect::<Vec<_>>()
            .join(", ");
        let mut reasons = Vec::new();
        if usage.targets.len() > 1 {
            reasons.push("多个目标字映射到同一槽位".to_string());
        }
        if !usage.literals.is_empty() {
            let literals = usage
                .literals
                .iter()
                .map(|character| format!("{character:?}"))
                .collect::<Vec<_>>()
                .join(", ");
            reasons.push(format!(
                "最终文本仍直接使用 {literals}（{}）",
                format_locations(&usage.literal_locations)
            ));
        }
        if !usage.markup_locations.is_empty() {
            reasons.push(format!(
                "最终文本仍有 <g:{index:04X}>（{}）",
                format_locations(&usage.markup_locations)
            ));
        }
        lines.push(format!(
            "槽位 0x{index:04X}: {mappings}; {}",
            reasons.join("；")
        ));
    }
    Err(ToolError(format!(
        "检测到字体槽位冲突，已拒绝构建：\n- {}",
        lines.join("\n- ")
    )))
}

fn format_locations(locations: &BTreeSet<String>) -> String {
    const DISPLAY_LIMIT: usize = 5;
    let mut rendered = locations
        .iter()
        .take(DISPLAY_LIMIT)
        .cloned()
        .collect::<Vec<_>>();
    if locations.len() > DISPLAY_LIMIT {
        rendered.push(format!("另 {} 处", locations.len() - DISPLAY_LIMIT));
    }
    rendered.join(", ")
}

fn format_missing_characters(characters: &BTreeSet<char>) -> String {
    let rendered = characters
        .iter()
        .map(|character| format!("U+{:04X} {:?}", *character as u32, character))
        .collect::<Vec<_>>()
        .join(", ");
    format!("translation contains unmapped characters: {rendered}")
}

fn format_missing_ttf(characters: &BTreeSet<char>) -> String {
    let rendered = characters
        .iter()
        .map(|character| format!("U+{:04X} {:?}", *character as u32, character))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "supplied TTF fonts do not contain {} required glyph(s): {rendered}",
        characters.len()
    )
}

fn discover_context_layout(context: &TranslationContext) -> ToolResult<SourceLayout> {
    if let Some(workspace) = &context.workspace {
        if !entries_belong_to_root(&workspace.source_root, &context.entries) {
            return Err(ToolError(format!(
                "translation workspace entries do not match source tree '{}'",
                workspace.source_root.display()
            )));
        }
        let (font_pairs, unified_output) = discover_font_pairs(&workspace.source_root)?;
        if font_pairs.is_empty() {
            return Err(ToolError(format!(
                "translation workspace contains no font_df_jpn.bin/.png under '{}'",
                workspace.source_root.display()
            )));
        }
        return Ok(SourceLayout {
            script_root: workspace.source_root.clone(),
            font_pairs,
            unified_output,
            workspace_root: Some(workspace.root.clone()),
        });
    }
    discover_source_layout(&context.json_input, &context.entries)
}

fn discover_source_layout(json_input: &Path, entries: &[TextEntry]) -> ToolResult<SourceLayout> {
    let base = if json_input.is_dir() {
        json_input.to_path_buf()
    } else {
        json_input
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    };
    let mut candidates = Vec::new();
    let mut current = Some(base.as_path());
    while let Some(path) = current {
        if path.is_dir() && !candidates.iter().any(|candidate| candidate == path) {
            candidates.push(path.to_path_buf());
        }
        if let Some(name) = path.file_name().and_then(|value| value.to_str()) {
            if let Some(stripped) = name.strip_suffix("_json") {
                let source_candidate = path.with_file_name(stripped);
                if !stripped.is_empty()
                    && source_candidate.is_dir()
                    && !candidates
                        .iter()
                        .any(|candidate| candidate == &source_candidate)
                {
                    candidates.push(source_candidate);
                }
            }
        }
        current = path.parent();
    }
    let mut fallback = None;
    for candidate in candidates {
        if !entries_belong_to_root(&candidate, entries) {
            continue;
        }
        if fallback.is_none() {
            fallback = Some(candidate.clone());
        }
        let (font_pairs, unified_output) = discover_font_pairs(&candidate)?;
        if !font_pairs.is_empty() {
            return Ok(SourceLayout {
                script_root: candidate,
                font_pairs,
                unified_output,
                workspace_root: None,
            });
        }
    }
    let script_root = fallback.ok_or_else(|| {
        ToolError(format!(
            "cannot locate source script directory for translation JSON '{}'",
            json_input.display()
        ))
    })?;
    let (font_pairs, unified_output) = discover_font_pairs(&script_root)?;
    if font_pairs.is_empty() {
        return Err(ToolError(format!(
            "cannot locate font_df_jpn.bin/.png beside source scripts near '{}'",
            script_root.display()
        )));
    }
    Ok(SourceLayout {
        script_root,
        font_pairs,
        unified_output,
        workspace_root: None,
    })
}

fn entries_belong_to_root(root: &Path, entries: &[TextEntry]) -> bool {
    entries.iter().all(|entry| {
        let Ok(relative) = safe_relative_path(&entry.file) else {
            return false;
        };
        let path = root.join(relative);
        path.is_file()
            && path.extension().is_some_and(|value| {
                value.eq_ignore_ascii_case("msb") || value.eq_ignore_ascii_case("scx")
            })
    })
}

fn count_source_script_files(root: &Path) -> ToolResult<usize> {
    let mut count = 0;
    for entry in fs::read_dir(root)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", root.display())))?
    {
        let entry = entry.map_err(|error| {
            ToolError(format!(
                "cannot read directory entry in '{}': {error}",
                root.display()
            ))
        })?;
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("chs"))
            {
                continue;
            }
            count += count_source_script_files(&path)?;
        } else if path.extension().is_some_and(|extension| {
            extension.eq_ignore_ascii_case("msb") || extension.eq_ignore_ascii_case("scx")
        }) {
            count += 1;
        }
    }
    Ok(count)
}

fn discover_font_pairs(root: &Path) -> ToolResult<(Vec<FontPair>, bool)> {
    let mut inside = Vec::new();
    collect_font_pairs(root, &mut inside)?;
    if !inside.is_empty() {
        return Ok((inside, true));
    }
    let parent = root.parent().unwrap_or_else(|| Path::new("."));
    let matching_name = root
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|name| {
            name.strip_suffix("_mes00")
                .or_else(|| name.strip_suffix("_script"))
        })
        .map(|prefix| format!("{prefix}_system_win"));
    if let Some(name) = &matching_name {
        let matching = parent.join(name);
        if matching.is_dir() {
            let mut pairs = Vec::new();
            collect_font_pairs(&matching, &mut pairs)?;
            if !pairs.is_empty() {
                return Ok((pairs, false));
            }
        }
    }
    let mut sibling_dirs = fs::read_dir(parent)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", parent.display())))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path != root)
        .collect::<Vec<_>>();
    sibling_dirs.sort();
    let mut pairs = Vec::new();
    for sibling in sibling_dirs {
        collect_font_pairs(&sibling, &mut pairs)?;
    }
    Ok((pairs, false))
}

fn collect_font_pairs(root: &Path, output: &mut Vec<FontPair>) -> ToolResult<()> {
    if root
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("chs"))
    {
        return Ok(());
    }
    let mut files = HashMap::new();
    let mut directories = Vec::new();
    for entry in fs::read_dir(root)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", root.display())))?
    {
        let entry = entry.map_err(|error| {
            ToolError(format!(
                "cannot read directory entry in '{}': {error}",
                root.display()
            ))
        })?;
        let path = entry.path();
        if path.is_dir() {
            directories.push(path);
        } else if let Some(name) = path.file_name().and_then(|value| value.to_str()) {
            files.insert(name.to_ascii_lowercase(), path);
        }
    }
    for (bin_name, png_name) in [
        ("font_df_jpn.bin", "font_df_jpn.png"),
        ("font2_df_jpn.bin", "font2_df_jpn.png"),
    ] {
        if let (Some(bin), Some(png)) = (files.get(bin_name), files.get(png_name)) {
            output.push(FontPair {
                bin: bin.clone(),
                png: png.clone(),
            });
        }
    }
    directories.sort();
    for directory in directories {
        collect_font_pairs(&directory, output)?;
    }
    Ok(())
}

fn inject_translation_json(
    script_root: &Path,
    json_input: &Path,
    output: &Path,
    entries: &[TextEntry],
    dictionary: &GlyphDictionary,
) -> ToolResult<InjectReport> {
    let parent = if json_input.is_dir() {
        json_input
    } else {
        json_input.parent().unwrap_or_else(|| Path::new("."))
    };
    let staging = parent.join(format!(".mpk-tool-json-stage-{}", std::process::id()));
    if staging.exists() {
        return Err(ToolError(format!(
            "temporary JSON staging path already exists: '{}'",
            staging.display()
        )));
    }
    fs::create_dir(&staging).map_err(|error| {
        ToolError(format!(
            "cannot create temporary JSON staging directory '{}': {error}",
            staging.display()
        ))
    })?;
    let staged_json = staging.join("translation.json");
    let result = write_entries(&staged_json, entries)
        .and_then(|_| inject_path(script_root, &staging, Some(output), dictionary));
    let _ = fs::remove_dir_all(&staging);
    result
}

fn build_font_pair_replacing(
    input_bin: &Path,
    input_png: &Path,
    ttf: &Path,
    donors: &[PathBuf],
    output_bin: &Path,
    output_png: &Path,
    targets: &HashSet<char>,
) -> ToolResult<crate::font::FontBuildReport> {
    let replace_existing = output_bin.exists() || output_png.exists();
    if !replace_existing {
        return build_font_pair_for_targets(
            input_bin, input_png, ttf, donors, output_bin, output_png, None, targets,
        );
    }
    let bin_name = output_bin
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ToolError(format!(
                "invalid font output path '{}'",
                output_bin.display()
            ))
        })?;
    let png_name = output_png
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ToolError(format!(
                "invalid font output path '{}'",
                output_png.display()
            ))
        })?;
    let parent = output_bin.parent().unwrap_or_else(|| Path::new("."));
    let temp_bin = parent.join(format!(".{bin_name}.tmp-{}", std::process::id()));
    let temp_png = parent.join(format!(".{png_name}.tmp-{}", std::process::id()));
    if temp_bin.exists() || temp_png.exists() {
        return Err(ToolError(format!(
            "temporary font output already exists beside '{}'",
            output_bin.display()
        )));
    }
    let result = build_font_pair_for_targets(
        input_bin, input_png, ttf, donors, &temp_bin, &temp_png, None, targets,
    );
    if let Err(error) = result {
        let _ = fs::remove_file(&temp_bin);
        let _ = fs::remove_file(&temp_png);
        return Err(error);
    }
    if let Err(error) = (|| -> ToolResult<()> {
        if output_bin.exists() {
            fs::remove_file(output_bin).map_err(|error| {
                ToolError(format!(
                    "cannot replace '{}': {error}",
                    output_bin.display()
                ))
            })?;
        }
        if output_png.exists() {
            fs::remove_file(output_png).map_err(|error| {
                ToolError(format!(
                    "cannot replace '{}': {error}",
                    output_png.display()
                ))
            })?;
        }
        fs::rename(&temp_bin, output_bin).map_err(|error| {
            ToolError(format!(
                "cannot finalize '{}': {error}",
                output_bin.display()
            ))
        })?;
        fs::rename(&temp_png, output_png).map_err(|error| {
            ToolError(format!(
                "cannot finalize '{}': {error}",
                output_png.display()
            ))
        })?;
        Ok(())
    })() {
        let _ = fs::remove_file(&temp_bin);
        let _ = fs::remove_file(&temp_png);
        return Err(error);
    }
    result
}

fn collect_json_files(root: &Path, output: &mut Vec<PathBuf>) -> ToolResult<()> {
    for entry in fs::read_dir(root)
        .map_err(|error| ToolError(format!("cannot list '{}': {error}", root.display())))?
    {
        let entry = entry.map_err(|error| {
            ToolError(format!(
                "cannot read directory entry in '{}': {error}",
                root.display()
            ))
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, output)?;
        } else if path
            .extension()
            .is_some_and(|value| value.eq_ignore_ascii_case("json"))
        {
            output.push(path);
        }
    }
    Ok(())
}

fn safe_relative_path(value: &str) -> ToolResult<PathBuf> {
    let path = PathBuf::from(value.replace('/', "\\"));
    if path.is_absolute() {
        return Err(ToolError(format!(
            "translation _file '{}' is absolute",
            value
        )));
    }
    for component in path.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(ToolError(format!(
                "translation _file '{}' escapes the source directory",
                value
            )));
        }
    }
    Ok(path)
}

fn dedup_paths(paths: &mut Vec<PathBuf>) {
    let mut seen = HashSet::new();
    paths.retain(|path| seen.insert(path.clone()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(message: String) -> TextEntry {
        TextEntry {
            file: "sample.msb".to_string(),
            index: 0,
            id: Some(0),
            offset: Some(16),
            size: Some(4),
            kind: Some("dialogue".to_string()),
            encoding: Some("glyph-index".to_string()),
            policy: Some("relocate".to_string()),
            name: None,
            scr_name: None,
            scr_msg: "原文".to_string(),
            message,
            message_parts: None,
            raw_body: Some("00".to_string()),
            extra: Default::default(),
        }
    }

    #[test]
    fn slot_conflict_rejects_a_still_used_carrier_character() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let carrier = dictionary.carrier_for('测').unwrap();
        let error =
            validate_slot_conflicts(&[entry(format!("测{carrier}"))], &dictionary).unwrap_err();
        assert!(error.0.contains("字体槽位冲突"));
        assert!(error.0.contains("'测'->'測'"));
        assert!(error.0.contains("最终文本仍直接使用 '測'"));
    }

    #[test]
    fn slot_conflict_rejects_direct_glyph_markup() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        let index = dictionary.index_for_translated_char('测').unwrap();
        let error = validate_slot_conflicts(&[entry(format!("测<g:{index:04X}>"))], &dictionary)
            .unwrap_err();
        assert!(error.0.contains(&format!("<g:{index:04X}>")));
    }

    #[test]
    fn slot_conflict_allows_an_unoccupied_target_slot() {
        let dictionary = GlyphDictionary::built_in().unwrap();
        validate_slot_conflicts(&[entry("测".to_string())], &dictionary).unwrap();
    }
}
