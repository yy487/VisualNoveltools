use anyhow::{bail, Context, Result};
use random_house_d88_tool::basic::BasicProgram;
use random_house_d88_tool::d88::D88Image;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const BASIC_ATTRIBUTE: u8 = 0x80;
const DATA: u8 = 0x84;
const PRINT: u8 = 0x91;
const COLOR: u8 = 0xCB;
const CLS: u8 = 0xCE;
const LOCATE: u8 = 0xD6;
const SYMBOL: [u8; 2] = [0xFF, 0xDF];
const Q_ASSIGN: [u8; 3] = [b'Q', b'$', 0xF1];
const SC_ASSIGN: [u8; 4] = [b'S', b'C', b'$', 0xF1];
const MES_ASSIGN: [u8; 5] = [b'M', b'E', b'S', b'$', 0xF1];

#[derive(Serialize)]
struct Report {
    format: &'static str,
    version: u32,
    source: String,
    source_sha256: String,
    basic_files: usize,
    basic_lines: usize,
    operation_counts: BTreeMap<String, usize>,
    files: Vec<FileReport>,
}

#[derive(Serialize)]
struct FileReport {
    name: String,
    lines: usize,
    layout_lines: Vec<LayoutLine>,
}

#[derive(Serialize)]
struct LayoutLine {
    line: u16,
    operations: Vec<&'static str>,
    raw_hex: String,
}

fn main() -> Result<()> {
    let source = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .context("usage: analyze_render_flow <source.d88> <output.json>")?;
    let output = env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .context("usage: analyze_render_flow <source.d88> <output.json>")?;
    if output.exists() {
        bail!("output already exists: {}", output.display());
    }

    let source_bytes = fs::read(&source)
        .with_context(|| format!("failed to read source D88 {}", source.display()))?;
    let image = D88Image::parse(source_bytes.clone()).context("failed to parse source D88")?;
    let mut operation_counts = BTreeMap::<String, usize>::new();
    let mut files = Vec::new();
    let mut basic_lines = 0usize;

    for file in &image.files {
        if file.attribute != BASIC_ATTRIBUTE {
            continue;
        }
        let program = BasicProgram::parse(&file.data)
            .with_context(|| format!("failed to parse BASIC file {}", file.name))?;
        basic_lines += program.lines.len();
        let mut layout_lines = Vec::new();
        for line in &program.lines {
            let content_end = line.body.iter().position(|byte| *byte == 0).unwrap();
            let content = &line.body[..content_end];
            let outside = outside_literals(content);
            let mut operations = Vec::new();
            mark_token(&outside, CLS, "CLS", &mut operations);
            mark_token(&outside, COLOR, "COLOR", &mut operations);
            mark_token(&outside, LOCATE, "LOCATE", &mut operations);
            mark_token(&outside, PRINT, "PRINT", &mut operations);
            mark_sequence(&outside, &SYMBOL, "SYMBOL", &mut operations);
            mark_token(&outside, DATA, "DATA", &mut operations);
            mark_sequence(&outside, &Q_ASSIGN, "Q_ASSIGN", &mut operations);
            mark_sequence(&outside, &SC_ASSIGN, "SC_ASSIGN", &mut operations);
            mark_sequence(&outside, &MES_ASSIGN, "MES_ASSIGN", &mut operations);
            if quoted_payloads(content)
                .iter()
                .any(|payload| payload.windows(3).any(|window| window == b"HIT"))
            {
                operations.push("HIT_WAIT");
            }
            if operations.is_empty() {
                continue;
            }
            operations.sort_unstable();
            operations.dedup();
            for operation in &operations {
                *operation_counts.entry((*operation).to_owned()).or_default() += 1;
            }
            layout_lines.push(LayoutLine {
                line: line.number,
                operations,
                raw_hex: hex_encode(content),
            });
        }
        files.push(FileReport {
            name: file.name.clone(),
            lines: program.lines.len(),
            layout_lines,
        });
    }

    let report = Report {
        format: "random-house-render-flow-analysis",
        version: 1,
        source: absolute_display(&source)?,
        source_sha256: format!("{:X}", Sha256::digest(&source_bytes)),
        basic_files: files.len(),
        basic_lines,
        operation_counts,
        files,
    };
    let bytes = serde_json::to_vec_pretty(&report)?;
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        bail!("output parent is not a directory: {}", parent.display());
    }
    fs::write(&output, [bytes.as_slice(), b"\n"].concat())
        .with_context(|| format!("failed to write {}", output.display()))?;
    println!(
        "RENDER_FLOW_ANALYSIS files={} lines={} layout_lines={} output={}",
        report.basic_files,
        report.basic_lines,
        report
            .files
            .iter()
            .map(|file| file.layout_lines.len())
            .sum::<usize>(),
        output.display()
    );
    Ok(())
}

fn outside_literals(content: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(content.len());
    let mut quoted = false;
    for byte in content.iter().copied() {
        if byte == b'"' {
            quoted = !quoted;
        } else if !quoted {
            output.push(byte);
        }
    }
    output
}

fn quoted_payloads(content: &[u8]) -> Vec<&[u8]> {
    let mut payloads = Vec::new();
    let mut opening = None;
    for (index, byte) in content.iter().copied().enumerate() {
        if byte != b'"' {
            continue;
        }
        if let Some(start) = opening.take() {
            payloads.push(&content[start..index]);
        } else {
            opening = Some(index + 1);
        }
    }
    if let Some(start) = opening {
        payloads.push(&content[start..]);
    }
    payloads
}

fn mark_token(outside: &[u8], token: u8, name: &'static str, operations: &mut Vec<&'static str>) {
    if outside.contains(&token) {
        operations.push(name);
    }
}

fn mark_sequence(
    outside: &[u8],
    sequence: &[u8],
    name: &'static str,
    operations: &mut Vec<&'static str>,
) {
    if outside
        .windows(sequence.len())
        .any(|window| window == sequence)
    {
        operations.push(name);
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn absolute_display(path: &Path) -> Result<String> {
    Ok(path
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", path.display()))?
        .display()
        .to_string())
}
