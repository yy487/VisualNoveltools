use crate::{arc, g24, msk};
use anyhow::{ensure, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct ReplacementRecord {
    pub source_png: String,
    pub target_entry: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub encoded_size: usize,
}

#[derive(Debug, Serialize)]
pub struct SkippedRecord {
    pub source_png: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct BuildReport {
    pub source_bg: String,
    pub source_data: String,
    pub output_bg: String,
    pub output_data: String,
    pub source_bg_sha256: String,
    pub source_data_sha256: String,
    pub output_bg_sha256: String,
    pub output_data_sha256: String,
    pub bg_verify: arc::VerifyResult,
    pub data_verify: arc::VerifyResult,
    pub replacements: Vec<ReplacementRecord>,
    pub skipped: Vec<SkippedRecord>,
}

pub fn build_patch(
    source_bg: &Path,
    source_data: &Path,
    edits_dir: &Path,
    output_dir: &Path,
) -> Result<BuildReport> {
    ensure!(
        source_bg.is_file(),
        "source Bg ARC does not exist: {}",
        source_bg.display()
    );
    ensure!(
        source_data.is_file(),
        "source data ARC does not exist: {}",
        source_data.display()
    );
    ensure!(
        edits_dir.is_dir(),
        "edit directory does not exist: {}",
        edits_dir.display()
    );
    fs::create_dir_all(output_dir)?;

    let bg_names: HashSet<String> = arc::read_index(source_bg)?
        .into_iter()
        .map(|entry| entry.name.to_ascii_uppercase())
        .collect();
    let data_names: HashSet<String> = arc::read_index(source_data)?
        .into_iter()
        .map(|entry| entry.name.to_ascii_uppercase())
        .collect();

    let mut pngs: Vec<PathBuf> = fs::read_dir(edits_dir)?
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
        })
        .collect();
    pngs.sort_by_key(|path| path.file_name().map(|name| name.to_ascii_lowercase()));

    let mut bg_replacements = HashMap::new();
    let mut data_replacements = HashMap::new();
    let mut records = Vec::new();
    let mut skipped = Vec::new();

    for png_path in pngs {
        let filename = png_path
            .file_name()
            .and_then(|value| value.to_str())
            .with_context(|| format!("non-Unicode edit filename: {}", png_path.display()))?;
        let upper = filename.to_ascii_uppercase();

        if upper == "INTRO_CBG.PNG" {
            skipped.push(SkippedRecord {
                source_png: filename.to_owned(),
                reason: "1448x1086 source canvas; INTRO_CBG1.png is the 640x480 game asset"
                    .to_owned(),
            });
            continue;
        }

        let is_mask = png_path
            .file_stem()
            .and_then(|value| value.to_str())
            .is_some_and(|stem| stem.to_ascii_uppercase().ends_with("_M"))
            || upper == "TITLE_PT_M_FULL_624X580.PNG";
        if is_mask {
            let target = if upper == "TITLE_PT_M_FULL_624X580.PNG" {
                "TITLE_PT_M.MSK".to_owned()
            } else {
                format!(
                    "{}.MSK",
                    png_path
                        .file_stem()
                        .and_then(|value| value.to_str())
                        .unwrap()
                )
            };
            ensure!(
                data_names.contains(&target.to_ascii_uppercase()),
                "mask edit has no target in data.arc: {filename} -> {target}"
            );
            ensure!(
                !data_replacements.contains_key(&target),
                "duplicate mask replacement target: {target}"
            );
            let image = image::open(&png_path)
                .with_context(|| format!("read edited mask: {}", png_path.display()))?;
            let gray = image.to_luma8();
            let dimensions = gray.dimensions();
            let template = arc::read_entry(source_data, &target)?;
            let kind = msk::classify_template(&template, &target, dimensions)?;
            let encoded = msk::encode(&image, kind)?;
            let decoded = msk::decode(&encoded, &target, Some(dimensions))?;
            ensure!(
                decoded.image == gray,
                "MSK verification failed after encode: {filename}"
            );
            records.push(ReplacementRecord {
                source_png: filename.to_owned(),
                target_entry: target.clone(),
                format: match kind {
                    msk::MskKind::TypeA { .. } => "MSK Type A".to_owned(),
                    msk::MskKind::Raw8 { .. } => "MSK raw 8-bit".to_owned(),
                    msk::MskKind::TitlePt => "MSK TITLE_PT 624x580 row-major".to_owned(),
                },
                width: dimensions.0,
                height: dimensions.1,
                encoded_size: encoded.len(),
            });
            data_replacements.insert(target, encoded);
            continue;
        }

        let target = if upper == "INTRO_CBG1.PNG" {
            "INTRO_CBG.G24".to_owned()
        } else {
            format!(
                "{}.G24",
                png_path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap()
            )
        };
        ensure!(
            bg_names.contains(&target.to_ascii_uppercase()),
            "image edit has no target in Bg.arc: {filename} -> {target}"
        );
        ensure!(
            !bg_replacements.contains_key(&target),
            "duplicate G24 replacement target: {target}"
        );
        let template = arc::read_entry(source_bg, &target)?;
        let header = g24::read_header(&template)?;
        let image = image::open(&png_path)
            .with_context(|| format!("read edited image: {}", png_path.display()))?;
        ensure!(
            image.width() == u32::from(header.width) && image.height() == u32::from(header.height),
            "G24 dimensions changed for {filename}: {}x{}, expected {}x{}",
            image.width(),
            image.height(),
            header.width,
            header.height
        );
        let encoded = g24::encode(&image, header.x, header.y)?;
        let (decoded_header, decoded) = g24::decode(&encoded)?;
        ensure!(
            (decoded_header.x, decoded_header.y) == (header.x, header.y)
                && decoded == image.to_rgb8(),
            "G24 verification failed after encode: {filename}"
        );
        records.push(ReplacementRecord {
            source_png: filename.to_owned(),
            target_entry: target.clone(),
            format: "G24 BGR bottom-up".to_owned(),
            width: image.width(),
            height: image.height(),
            encoded_size: encoded.len(),
        });
        bg_replacements.insert(target, encoded);
    }

    ensure!(
        !bg_replacements.is_empty(),
        "no G24 replacements found in edits directory"
    );
    ensure!(
        !data_replacements.is_empty(),
        "no MSK replacements found in edits directory"
    );

    let output_bg = output_dir.join("bg_chs.arc");
    let output_data = output_dir.join("data_chs.arc");
    arc::repack_with_replacements(source_bg, &output_bg, &bg_replacements)?;
    arc::repack_with_replacements(source_data, &output_data, &data_replacements)?;
    let bg_verify = arc::verify_repack(source_bg, &output_bg, &bg_replacements)?;
    let data_verify = arc::verify_repack(source_data, &output_data, &data_replacements)?;

    let report = BuildReport {
        source_bg: source_bg.display().to_string(),
        source_data: source_data.display().to_string(),
        output_bg: output_bg.display().to_string(),
        output_data: output_data.display().to_string(),
        source_bg_sha256: sha256_file(source_bg)?,
        source_data_sha256: sha256_file(source_data)?,
        output_bg_sha256: sha256_file(&output_bg)?,
        output_data_sha256: sha256_file(&output_data)?,
        bg_verify,
        data_verify,
        replacements: records,
        skipped,
    };
    fs::write(
        output_dir.join("build_report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(report)
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut digest = Sha256::new();
    let mut buffer = vec![0u8; 1024 * 1024];
    loop {
        let amount = reader.read(&mut buffer)?;
        if amount == 0 {
            break;
        }
        digest.update(&buffer[..amount]);
    }
    Ok(format!("{:X}", digest.finalize()))
}
